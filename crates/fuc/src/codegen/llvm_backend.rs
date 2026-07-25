// LLVM Backend - requires inkwell crate
#![cfg(feature = "llvm")]
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::builder::Builder;
use inkwell::values::{BasicValueEnum, IntValue, PointerValue, FunctionValue, PhiValue};
use inkwell::types::{BasicType, BasicTypeEnum, StructType, BasicMetadataTypeEnum};
use inkwell::IntPredicate;
use inkwell::FloatPredicate;
use inkwell::AddressSpace;
use crate::ir::{self, *};
use crate::codegen::{Backend, CodegenConfig, CodegenError};
use std::collections::HashMap;

pub struct LlvmBackend<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    config: CodegenConfig,
    value_map: HashMap<String, BasicValueEnum<'ctx>>,
    struct_types: HashMap<String, StructType<'ctx>>,
    #[allow(dead_code)]
    string_globals: HashMap<String, PointerValue<'ctx>>,
    current_function: Option<FunctionValue<'ctx>>,
    pending_phis: Vec<(PhiValue<'ctx>, Vec<(TypedValue, usize)>)>,
}

impl<'ctx> LlvmBackend<'ctx> {
    pub fn new(context: &'ctx Context, name: &str, config: &CodegenConfig) -> Self {
        let module = context.create_module(name);
        let builder = context.create_builder();
        Self {
            context,
            module,
            builder,
            config: config.clone(),
            value_map: HashMap::new(),
            struct_types: HashMap::new(),
            string_globals: HashMap::new(),
            current_function: None,
            pending_phis: Vec::new(),
        }
    }

    /// Compile an entire IR module.
    pub fn compile_ir_module(&mut self, ir_module: &IrModule) -> Result<(), CodegenError> {
        // Declare struct types first
        for sd in &ir_module.structs {
            self.declare_struct(sd)?;
        }

        // Declare extern functions
        for ext in &ir_module.externs {
            self.declare_extern(ext)?;
        }

        // Compile functions
        for func in &ir_module.functions {
            self.compile_function(func)?;
        }

        Ok(())
    }

    /// Declare a struct type in LLVM.
    fn declare_struct(&mut self, sd: &IrStructDef) -> Result<(), CodegenError> {
        let field_types: Vec<BasicTypeEnum<'ctx>> = sd.fields.iter()
            .map(|(_, ty)| self.ir_type_to_llvm(ty))
            .collect::<Result<Vec<_>, _>>()?;
        let struct_type = self.context.struct_type(&field_types, false);
        self.struct_types.insert(sd.name.clone(), struct_type);
        Ok(())
    }

    /// Declare an extern function.
    fn declare_extern(&mut self, ext: &IrExtern) -> Result<(), CodegenError> {
        let ret_type = self.ir_type_to_llvm(&ext.return_type)?;
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = ext.params.iter()
            .map(|t| self.ir_type_to_llvm(t).map(|t| t.into()))
            .collect::<Result<Vec<_>, _>>()?;
        let fn_type = ret_type.fn_type(&param_types, false);
        self.module.add_function(&ext.name, fn_type, None);
        Ok(())
    }

    /// Compile a single IR function.
    pub fn compile_function(&mut self, func: &IrFunction) -> Result<FunctionValue<'ctx>, CodegenError> {
        self.value_map.clear();
        self.pending_phis.clear();

        let ret_type = self.ir_type_to_llvm(&func.return_type)?;
        let param_types: Vec<BasicMetadataTypeEnum<'ctx>> = func.params.iter()
            .map(|(_, ty)| self.ir_type_to_llvm(ty).map(|t| t.into()))
            .collect::<Result<Vec<_>, _>>()?;
        let fn_type = if func.return_type == Type::Void {
            self.context.void_type().fn_type(&param_types, false)
        } else {
            ret_type.fn_type(&param_types, false)
        };
        let llvm_func = self.module.add_function(&func.name, fn_type, None);
        self.current_function = Some(llvm_func);

        // Create LLVM basic blocks
        let mut llvm_blocks: Vec<inkwell::basic_block::BasicBlock<'ctx>> = Vec::new();
        for block in &func.blocks {
            let bb = self.context.append_basic_block(llvm_func, &block.label);
            llvm_blocks.push(bb);
        }

        // Compile each block
        for (i, block) in func.blocks.iter().enumerate() {
            self.builder.position_at_end(llvm_blocks[i]);

            for instr in &block.instrs {
                self.compile_instruction(instr)?;
            }

            self.compile_terminator(&block.terminator, &llvm_blocks)?;
        }

        // Finalize phi nodes (add incoming values now that all blocks are compiled)
        self.finalize_phis(&llvm_blocks)?;

        Ok(llvm_func)
    }

    /// Finalize pending phi nodes by adding incoming values.
    fn finalize_phis(
        &mut self,
        llvm_blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
    ) -> Result<(), CodegenError> {
        let pending = std::mem::take(&mut self.pending_phis);
        for (phi, incoming) in pending {
            for (val, block_id) in incoming {
                let llvm_val = self.resolve_value(&val)?;
                let bb = llvm_blocks[block_id];
                phi.add_incoming(&[(&llvm_val, bb)]);
            }
        }
        Ok(())
    }

    /// Compile a single IR instruction.
    fn compile_instruction(&mut self, instr: &Instruction) -> Result<(), CodegenError> {
        match instr {
            Instruction::BinaryOperation { dest, op, op1, op2 } => {
                let left = self.resolve_value(op1)?;
                let right = self.resolve_value(op2)?;
                let result = match (left, right) {
                    (BasicValueEnum::IntValue(l), BasicValueEnum::IntValue(r)) => {
                        let val = match op {
                            BinaryOp::Add => self.builder.build_int_add(l, r, "add"),
                            BinaryOp::Sub => self.builder.build_int_sub(l, r, "sub"),
                            BinaryOp::Mul => self.builder.build_int_mul(l, r, "mul"),
                            BinaryOp::Div => self.builder.build_int_signed_div(l, r, "div"),
                            BinaryOp::Mod => self.builder.build_int_signed_rem(l, r, "mod"),
                            BinaryOp::Eq => self.builder.build_int_compare(IntPredicate::EQ, l, r, "eq"),
                            BinaryOp::Neq => self.builder.build_int_compare(IntPredicate::NE, l, r, "neq"),
                            BinaryOp::Lt => self.builder.build_int_compare(IntPredicate::SLT, l, r, "lt"),
                            BinaryOp::Gt => self.builder.build_int_compare(IntPredicate::SGT, l, r, "gt"),
                            BinaryOp::Le => self.builder.build_int_compare(IntPredicate::SLE, l, r, "le"),
                            BinaryOp::Ge => self.builder.build_int_compare(IntPredicate::SGE, l, r, "ge"),
                            BinaryOp::And => {
                                let l_int_ty = l.get_type();
                                let l_bool = self.builder.build_int_compare(IntPredicate::NE, l, l_int_ty.const_int(0, false), "l_bool")
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                                let r_bool = self.builder.build_int_compare(IntPredicate::NE, r, r.get_type().const_int(0, false), "r_bool")
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                                let and_val = self.builder.build_and(l_bool, r_bool, "and")
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                                self.builder.build_int_z_extend(and_val, l_int_ty, "and_ext")
                            }
                            BinaryOp::Or => {
                                let l_int_ty = l.get_type();
                                let l_bool = self.builder.build_int_compare(IntPredicate::NE, l, l_int_ty.const_int(0, false), "l_bool")
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                                let r_bool = self.builder.build_int_compare(IntPredicate::NE, r, r.get_type().const_int(0, false), "r_bool")
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                                let or_val = self.builder.build_or(l_bool, r_bool, "or")
                                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                                self.builder.build_int_z_extend(or_val, l_int_ty, "or_ext")
                            }
                        };
                        val.map(BasicValueEnum::from).map_err(|e| CodegenError::LlvmError(e.to_string()))?
                    }
                    (BasicValueEnum::FloatValue(l), BasicValueEnum::FloatValue(r)) => {
                        let val: BasicValueEnum<'ctx> = match op {
                            BinaryOp::Add => self.builder.build_float_add(l, r, "fadd")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Sub => self.builder.build_float_sub(l, r, "fsub")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Mul => self.builder.build_float_mul(l, r, "fmul")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Div => self.builder.build_float_div(l, r, "fdiv")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Mod => self.builder.build_float_rem(l, r, "fmod")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Eq => self.builder.build_float_compare(FloatPredicate::OEQ, l, r, "feq")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Neq => self.builder.build_float_compare(FloatPredicate::ONE, l, r, "fne")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Lt => self.builder.build_float_compare(FloatPredicate::OLT, l, r, "flt")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Gt => self.builder.build_float_compare(FloatPredicate::OGT, l, r, "fgt")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Le => self.builder.build_float_compare(FloatPredicate::OLE, l, r, "fle")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::Ge => self.builder.build_float_compare(FloatPredicate::OGE, l, r, "fge")
                                .map_err(|e| CodegenError::LlvmError(e.to_string()))?.into(),
                            BinaryOp::And | BinaryOp::Or => return Err(CodegenError::Unsupported("Logical ops on float not supported".to_string())),
                        };
                        val
                    }
                    _ => return Err(CodegenError::Unsupported("Binary op on non-int types".to_string())),
                };
                self.store_value(dest, result);
            }
            Instruction::Call { dest, func_name, args } => {
                let llvm_func = self.module.get_function(func_name)
                    .ok_or_else(|| CodegenError::LlvmError(format!("Undefined function: {}", func_name)))?;
                let llvm_args: Vec<inkwell::values::BasicMetadataValueEnum<'ctx>> = args.iter()
                    .map(|a| self.resolve_value(a).map(|v| v.into()))
                    .collect::<Result<Vec<_>, _>>()?;
                let call_result = self.builder.build_call(llvm_func, &llvm_args, "call")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                if let Some(d) = dest {
                    if let Some(val) = call_result.try_as_basic_value().basic() {
                        self.store_value(d, val);
                    }
                }
            }
            Instruction::Load { dest, src } => {
                let ptr = self.resolve_address(src)?;
                let pointee_ty = self.address_pointee_type(src)?;
                let loaded = self.builder.build_load(pointee_ty, ptr, "load")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.store_value(dest, loaded);
            }
            Instruction::Store { dest, val } => {
                let ptr = self.resolve_address(dest)?;
                let llvm_val = self.resolve_value(val)?;
                self.builder.build_store(ptr, llvm_val)
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
            Instruction::GetElementPtr { dest, base_ptr, index, element_ty } => {
                let ptr_val = self.resolve_value(base_ptr)?;
                let ptr = ptr_val.into_pointer_value();
                let idx = self.resolve_value(index)?;
                let idx_val = idx.into_int_value();
                let elem_llvm_ty = self.ir_type_to_llvm(element_ty)?;
                unsafe {
                    let gep = self.builder.build_gep(elem_llvm_ty, ptr, &[idx_val], "gep")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.store_value(dest, gep.into());
                }
            }
            Instruction::GetFieldPtr { dest, base_ptr, field_index, field_ty, struct_name: _ } => {
                let ptr_val = self.resolve_value(base_ptr)?;
                let ptr = ptr_val.into_pointer_value();
                let zero = self.context.i32_type().const_int(0, false);
                let idx = self.context.i32_type().const_int(*field_index as u64, false);
                let field_llvm_ty = self.ir_type_to_llvm(field_ty)?;
                unsafe {
                    let gep = self.builder.build_gep(field_llvm_ty, ptr, &[zero, idx], "field_ptr")
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                    self.store_value(dest, gep.into());
                }
            }
            Instruction::Alloca { dest, ty } => {
                let llvm_ty = self.ir_type_to_llvm(ty)?;
                let alloca = self.builder.build_alloca(llvm_ty, "alloca")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.store_value(dest, alloca.into());
            }
            Instruction::Copy { dest, src } => {
                let val = self.resolve_value(src)?;
                self.store_value(dest, val);
            }
            Instruction::UnaryNot { dest, operand } => {
                let val = self.resolve_value(operand)?;
                let int_val = val.into_int_value();
                let zero = self.context.i64_type().const_int(0, false);
                let cmp = self.builder.build_int_compare(IntPredicate::EQ, int_val, zero, "not")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                let ext = self.builder.build_int_z_extend(cmp, self.context.i64_type(), "not_ext")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.store_value(dest, ext.into());
            }
            Instruction::Phi { dest, incoming } => {
                let llvm_ty = self.ir_type_to_llvm(&dest.ty)?;
                let phi = self.builder.build_phi(llvm_ty, "phi")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.store_value(dest, phi.as_basic_value());
                // Defer incoming value resolution until finalize_phis
                self.pending_phis.push((phi, incoming.clone()));
            }
            Instruction::MakeClosure { dest, func_name, captured: _ } => {
                // Simplified: store function pointer as first capture
                let llvm_func = self.module.get_function(func_name)
                    .ok_or_else(|| CodegenError::LlvmError(format!("Undefined function: {}", func_name)))?;
                let ptr = llvm_func.as_global_value().as_pointer_value();
                self.store_value(dest, ptr.into());
            }
            Instruction::Comment(_) => {
                // No-op
            }
            Instruction::GetAddress { dest, var_name, is_mutable: _ } => {
                if let Some(val) = self.value_map.get(var_name) {
                    if let BasicValueEnum::PointerValue(ptr) = val {
                        self.value_map.insert(dest.clone(), (*ptr).into());
                    }
                }
            }
        }
        Ok(())
    }

    /// Compile a terminator instruction.
    fn compile_terminator(
        &mut self,
        term: &Terminator,
        llvm_blocks: &[inkwell::basic_block::BasicBlock<'ctx>],
    ) -> Result<(), CodegenError> {
        match term {
            Terminator::Jump(target) => {
                self.builder.build_unconditional_branch(llvm_blocks[*target])
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
            Terminator::ConditionalJump { cond, then_block, else_block } => {
                let cond_val = self.resolve_value(cond)?;
                let cond_int = cond_val.into_int_value();
                let is_true = self.builder.build_int_compare(
                    IntPredicate::NE,
                    cond_int,
                    self.context.i64_type().const_int(0, false),
                    "cond",
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                self.builder.build_conditional_branch(
                    is_true,
                    llvm_blocks[*then_block],
                    llvm_blocks[*else_block],
                ).map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
            Terminator::Return(val) => {
                if let Some(v) = val {
                    let llvm_val = self.resolve_value(v)?;
                    self.builder.build_return(Some(&llvm_val))
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                } else {
                    self.builder.build_return(None)
                        .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
                }
            }
            Terminator::Unreachable => {
                self.builder.build_unreachable()
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Resolve a TypedValue to an LLVM BasicValueEnum.
    fn resolve_value(&self, tv: &TypedValue) -> Result<BasicValueEnum<'ctx>, CodegenError> {
        match &tv.val {
            Value::IntConst(n) => {
                let ty = self.ir_type_to_llvm(&tv.ty)?;
                Ok(ty.into_int_type().const_int(*n as u64, true).into())
            }
            Value::BoolConst(b) => {
                Ok(self.context.bool_type().const_int(*b as u64, false).into())
            }
            Value::StringConst(s) => {
                let global = self.module.add_global(
                    self.context.i8_type().array_type(s.len() as u32),
                    None,
                    &s,
                );
                let ptr = global.as_pointer_value();
                Ok(ptr.into())
            }
            Value::FloatConst(f) => {
                Ok(self.context.f64_type().const_float(*f).into())
            }
            Value::Variable(name) => {
                self.value_map.get(name)
                    .cloned()
                    .ok_or_else(|| CodegenError::LlvmError(format!("Undefined value: {}", name)))
            }
            Value::Temp(n) => {
                let key = format!("%{}", n);
                self.value_map.get(&key)
                    .cloned()
                    .ok_or_else(|| CodegenError::LlvmError(format!("Undefined value: {}", key)))
            }
        }
    }

    /// Resolve an Address to an LLVM PointerValue.
    fn resolve_address(&self, addr: &Address) -> Result<PointerValue<'ctx>, CodegenError> {
        match addr {
            Address::Variable { name, .. } => {
                self.value_map.get(name)
                    .and_then(|v| if let BasicValueEnum::PointerValue(p) = v { Some(*p) } else { None })
                    .ok_or_else(|| CodegenError::LlvmError(format!("Undefined address: {}", name)))
            }
            Address::Pointer { val, .. } => {
                let ptr_val = self.resolve_value(val)?;
                Ok(ptr_val.into_pointer_value())
            }
            Address::Element { base, index, element_ty } => {
                let base_ptr = self.resolve_address(base)?;
                let idx_val = self.resolve_value(index)?;
                let idx_int = idx_val.into_int_value();
                let elem_llvm_ty = self.ir_type_to_llvm(element_ty)?;
                let gep = self.build_index_gep(base_ptr, idx_int, elem_llvm_ty)?;
                Ok(gep)
            }
            Address::Field { base, field_index, field_ty, struct_name } => {
                let base_ptr = self.resolve_address(base)?;
                let gep = self.build_field_gep(base_ptr, *field_index, field_ty, struct_name)?;
                Ok(gep)
            }
        }
    }

    /// Build a GEP for array/slice element access.
    fn build_index_gep(
        &self,
        base_ptr: PointerValue<'ctx>,
        index: IntValue<'ctx>,
        element_ty: BasicTypeEnum<'ctx>,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        unsafe {
            self.builder.build_gep(element_ty, base_ptr, &[index], "elem_gep")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))
        }
    }

    /// Build a GEP for struct field access.
    fn build_field_gep(
        &self,
        base_ptr: PointerValue<'ctx>,
        field_index: usize,
        field_ty: &ir::Type,
        _struct_name: &str,
    ) -> Result<PointerValue<'ctx>, CodegenError> {
        let zero = self.context.i32_type().const_int(0, false);
        let idx = self.context.i32_type().const_int(field_index as u64, false);
        let field_llvm_ty = self.ir_type_to_llvm(field_ty)?;
        unsafe {
            self.builder.build_gep(field_llvm_ty, base_ptr, &[zero, idx], "field_gep")
                .map_err(|e| CodegenError::LlvmError(e.to_string()))
        }
    }

    /// Coerce an LLVM value to i1 (bool).
    #[allow(dead_code)]
    fn coerce_to_bool(&self, val: BasicValueEnum<'ctx>) -> Result<IntValue<'ctx>, CodegenError> {
        match val {
            BasicValueEnum::IntValue(i) => {
                let zero = i.get_type().const_int(0, false);
                self.builder.build_int_compare(IntPredicate::NE, i, zero, "to_bool")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }
            BasicValueEnum::FloatValue(f) => {
                let zero = f.get_type().const_float(0.0);
                self.builder.build_float_compare(FloatPredicate::ONE, f, zero, "to_bool")
                    .map_err(|e| CodegenError::LlvmError(e.to_string()))
            }
            _ => Err(CodegenError::Unsupported("Cannot coerce value to bool".to_string())),
        }
    }

    /// Coerce an i1 to i64 (for int-typed bools).
    #[allow(dead_code)]
    fn coerce_int_to_bool(&self, val: IntValue<'ctx>) -> Result<IntValue<'ctx>, CodegenError> {
        self.builder.build_int_z_extend(val, self.context.i64_type(), "bool_to_int")
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Get the pointee type for an Address.
    fn address_pointee_type(&self, addr: &Address) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        match addr {
            Address::Variable { ty, .. } => self.ir_type_to_llvm(ty),
            Address::Pointer { pointed_to_ty, .. } => self.ir_type_to_llvm(pointed_to_ty),
            Address::Element { element_ty, .. } => self.ir_type_to_llvm(element_ty),
            Address::Field { field_ty, .. } => self.ir_type_to_llvm(field_ty),
        }
    }

    /// Declare a global string constant and return its pointer.
    #[allow(dead_code)]
    fn declare_global_string(&mut self, name: &str, value: &str) -> Result<PointerValue<'ctx>, CodegenError> {
        if let Some(ptr) = self.string_globals.get(name) {
            return Ok(*ptr);
        }
        let global = self.module.add_global(
            self.context.i8_type().array_type(value.len() as u32 + 1),
            None,
            name,
        );
        let chars: Vec<IntValue<'ctx>> = value.bytes()
            .chain(std::iter::once(0u8))
            .map(|b| self.context.i8_type().const_int(b as u64, false))
            .collect();
        let const_array_val = self.context.i8_type().const_array(&chars);
        global.set_initializer(&const_array_val);
        let ptr = global.as_pointer_value();
        self.string_globals.insert(name.to_string(), ptr);
        Ok(ptr)
    }

    /// Store a value in the value map.
    fn store_value(&mut self, tv: &TypedValue, val: BasicValueEnum<'ctx>) {
        let key = match &tv.val {
            Value::Temp(n) => format!("%{}", n),
            Value::Variable(n) => n.clone(),
            _ => return,
        };
        self.value_map.insert(key, val);
    }

    /// Convert an IR type to an LLVM type.
    fn ir_type_to_llvm(&self, ty: &Type) -> Result<BasicTypeEnum<'ctx>, CodegenError> {
        match ty {
            Type::Int => Ok(self.context.i64_type().into()),
            Type::Bool => Ok(self.context.bool_type().into()),
            Type::Float => Ok(self.context.f64_type().into()),
            Type::Void => Err(CodegenError::Unsupported("Void type cannot be used as a value type".to_string())),
            Type::String | Type::Slice(_) => Ok(self.context.ptr_type(AddressSpace::default()).into()),
            Type::Pointer(inner) => {
                let _inner_ty = self.ir_type_to_llvm(inner)?;
                Ok(self.context.ptr_type(AddressSpace::default()).into())
            }
            Type::Array(elem, len) => {
                let elem_ty = self.ir_type_to_llvm(elem)?;
                Ok(elem_ty.array_type(*len as u32).into())
            }
            Type::Struct(name) => {
                self.struct_types.get(name)
                    .cloned()
                    .map(|t| t.into())
                    .ok_or_else(|| CodegenError::LlvmError(format!("Undefined struct: {}", name)))
            }
            _ => Ok(self.context.i64_type().into()), // Default fallback
        }
    }

    /// Write the module to a file.
    pub fn write_to_file(&self, path: &str) -> Result<(), CodegenError> {
        self.module.print_to_file(path)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }

    /// Write the module to memory as LLVM IR text.
    pub fn write_to_string(&self) -> String {
        self.module.print_to_string().to_string()
    }

    /// Write the module to an object file.
    pub fn write_object_file(&self, path: &str) -> Result<(), CodegenError> {
        // Use LLVM's TargetMachine for native object emission
        use inkwell::targets::{Target, FileType, RelocMode, CodeModel};
        use inkwell::OptimizationLevel;

        Target::initialize_native(&inkwell::targets::InitializationConfig::default())
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;

        let triple = inkwell::targets::TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple)
            .map_err(|e| CodegenError::LlvmError(e.to_string()))?;
        let opt_level = match self.config.optimization_level {
            0 => OptimizationLevel::None,
            1 => OptimizationLevel::Less,
            2 => OptimizationLevel::Default,
            _ => OptimizationLevel::Aggressive,
        };
        let target_machine = target.create_target_machine(
            &triple,
            "generic",
            "",
            opt_level,
            RelocMode::Default,
            CodeModel::Default,
        ).ok_or_else(|| CodegenError::LlvmError("Failed to create target machine".to_string()))?;

        target_machine.write_to_file(&self.module, FileType::Object, std::path::Path::new(path))
            .map_err(|e| CodegenError::LlvmError(e.to_string()))
    }
}

impl<'ctx> Backend for LlvmBackend<'ctx> {
    fn compile_module(&mut self, module: &IrModule) -> Result<Vec<u8>, CodegenError> {
        // Compile to memory
        self.compile_ir_module(module)?;
        // Return empty bytes for now - object file emission is separate
        Ok(Vec::new())
    }
}