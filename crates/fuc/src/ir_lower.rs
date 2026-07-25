//! IR Lowering: TypedProgram → IrModule
//! Converts the typed AST into the compiler's intermediate representation.

use crate::ir::*;
use crate::sema::*;
use std::collections::HashMap;

/// IR builder with register counter and block management.
pub struct IrBuilder {
    reg_counter: usize,
    blocks: Vec<BasicBlock>,
    current_block: usize,
    string_counter: usize,
    global_strings: Vec<IrGlobalString>,
}

impl IrBuilder {
    pub fn new() -> Self {
        let entry = BasicBlock {
            label: "entry".to_string(),
            instrs: Vec::new(),
            terminator: Terminator::return_(None),
        };
        Self {
            reg_counter: 0,
            blocks: vec![entry],
            current_block: 0,
            string_counter: 0,
            global_strings: Vec::new(),
        }
    }

    fn next_reg(&mut self) -> usize {
        let r = self.reg_counter;
        self.reg_counter += 1;
        r
    }

    fn next_string_name(&mut self) -> String {
        let n = self.string_counter;
        self.string_counter += 1;
        format!(".str.{}", n)
    }

    fn emit(&mut self, instr: Instruction) {
        self.blocks[self.current_block].instrs.push(instr);
    }

    fn set_terminator(&mut self, term: Terminator) {
        self.blocks[self.current_block].terminator = term;
    }

    fn new_block(&mut self, label: String) -> usize {
        let id = self.blocks.len();
        self.blocks.push(BasicBlock {
            label,
            instrs: Vec::new(),
            terminator: Terminator::return_(None),
        });
        id
    }

    fn switch_to_block(&mut self, id: usize) {
        self.current_block = id;
    }

    fn temp_val(&self, reg: usize, ty: Type) -> TypedValue {
        TypedValue { val: Value::Temp(reg), ty }
    }

    fn int_val(&self, n: i64) -> TypedValue {
        TypedValue { val: Value::IntConst(n), ty: Type::Int }
    }

    fn bool_val(&self, b: bool) -> TypedValue {
        TypedValue { val: Value::BoolConst(b), ty: Type::Bool }
    }

    fn string_val(&self, s: String) -> TypedValue {
        TypedValue { val: Value::StringConst(s), ty: Type::String }
    }

    fn var_val(&self, name: &str, ty: Type) -> TypedValue {
        TypedValue { val: Value::Variable(name.to_string()), ty }
    }

    fn float_val(&self, n: f64) -> TypedValue {
        TypedValue { val: Value::FloatConst(n), ty: Type::Float }
    }

    /// Lower a typed program into an IR module.
    pub fn lower_program(&mut self, prog: &TypedProgram) -> IrModule {
        let mut functions: Vec<IrFunction> = Vec::new();
        let externs: Vec<IrExtern> = Vec::new();
        let mut structs: Vec<IrStructDef> = Vec::new();

        // Lower structs
        for sd in &prog.structs {
            structs.push(IrStructDef {
                name: sd.name.clone(),
                fields: sd.fields.clone(),
            });
        }

        // Lower functions
        for func in &prog.functions {
            let ir_func = self.lower_function(func);
            functions.push(ir_func);
        }

        IrModule {
            functions,
            externs,
            structs,
            enums: Vec::new(),
            global_strings: std::mem::take(&mut self.global_strings),
        }
    }

    fn lower_function(&mut self, func: &TypedFunction) -> IrFunction {
        // Reset builder state
        self.reg_counter = 0;
        self.blocks.clear();
        self.current_block = 0;

        let entry = BasicBlock {
            label: "entry".to_string(),
            instrs: Vec::new(),
            terminator: Terminator::return_(None),
        };
        self.blocks.push(entry);

        // Allocate locals for parameters
        let mut var_map: HashMap<String, TypedValue> = HashMap::new();
        for (name, ty) in &func.params {
            let reg = self.next_reg();
            let slot = self.temp_val(reg, ty.clone());
            self.emit(Instruction::Alloca {
                dest: slot.clone(),
                ty: ty.clone(),
            });
            // Store incoming argument value into the allocated slot
            self.emit(Instruction::Store {
                dest: Address::Pointer {
                    val: slot.clone(),
                    pointed_to_ty: ty.clone(),
                },
                val: TypedValue {
                    val: Value::Variable(name.clone()),
                    ty: ty.clone(),
                },
            });
            var_map.insert(name.clone(), slot);
        }

        // Lower the function body
        self.lower_block(&func.body, &mut var_map, &func.return_type);

        // Ensure terminator
        if self.blocks[self.current_block].instrs.is_empty()
            && matches!(self.blocks[self.current_block].terminator, Terminator::Return(None))
        {
            // Already has a default return terminator
        }

        IrFunction {
            name: func.name.clone(),
            params: func.params.clone(),
            return_type: func.return_type.clone(),
            blocks: self.blocks.clone(),
            entry_block: 0,
        }
    }

    fn lower_block(
        &mut self,
        stmts: &[TypedStatement],
        var_map: &mut HashMap<String, TypedValue>,
        return_type: &Type,
    ) {
        for stmt in stmts {
            self.lower_statement(stmt, var_map, return_type);
        }
    }

    fn lower_statement(
        &mut self,
        stmt: &TypedStatement,
        var_map: &mut HashMap<String, TypedValue>,
        return_type: &Type,
    ) {
        match stmt {
            TypedStatement::Let { name, value, ty } => {
                let val = self.lower_expression(value, var_map);
                let reg = self.next_reg();
                let slot = self.temp_val(reg, ty.clone());
                self.emit(Instruction::Alloca { dest: slot.clone(), ty: ty.clone() });
                self.emit(Instruction::Store {
                    dest: Address::Pointer { val: slot.clone(), pointed_to_ty: ty.clone() },
                    val,
                });
                var_map.insert(name.clone(), slot);
            }
            TypedStatement::Assignment { target, value } => {
                let val = self.lower_expression(value, var_map);
                let addr = self.lower_lvalue(target, var_map);
                self.emit(Instruction::Store { dest: addr, val });
            }
            TypedStatement::Expression(expr) => {
                self.lower_expression(expr, var_map);
            }
            TypedStatement::Return(Some(expr)) => {
                let val = self.lower_expression(expr, var_map);
                self.set_terminator(Terminator::return_(Some(val)));
            }
            TypedStatement::Return(None) => {
                self.set_terminator(Terminator::return_(None));
            }
            TypedStatement::If { cond, then_block, else_block } => {
                let cond_val = self.lower_expression(cond, var_map);

                let then_id = self.new_block("then".to_string());
                let else_id = self.new_block("else".to_string());
                let merge_id = self.new_block("merge".to_string());

                self.set_terminator(Terminator::ConditionalJump {
                    cond: cond_val,
                    then_block: then_id,
                    else_block: else_id,
                });

                // Then block
                self.switch_to_block(then_id);
                self.lower_block(then_block, var_map, return_type);
                self.set_terminator(Terminator::Jump(merge_id));

                // Else block
                self.switch_to_block(else_id);
                if let Some(else_b) = else_block {
                    self.lower_block(else_b, var_map, return_type);
                }
                self.set_terminator(Terminator::Jump(merge_id));

                // Merge
                self.switch_to_block(merge_id);
            }
            TypedStatement::While { cond, body } => {
                let header_id = self.new_block("while_header".to_string());
                let body_id = self.new_block("while_body".to_string());
                let exit_id = self.new_block("while_exit".to_string());

                self.set_terminator(Terminator::Jump(header_id));

                // Header
                self.switch_to_block(header_id);
                let cond_val = self.lower_expression(cond, var_map);
                self.set_terminator(Terminator::ConditionalJump {
                    cond: cond_val,
                    then_block: body_id,
                    else_block: exit_id,
                });

                // Body
                self.switch_to_block(body_id);
                self.lower_block(body, var_map, return_type);
                self.set_terminator(Terminator::Jump(header_id));

                // Exit
                self.switch_to_block(exit_id);
            }
            TypedStatement::For { var, iter, body } => {
                let iter_val = self.lower_expression(iter, var_map);

                let header_id = self.new_block("for_header".to_string());
                let body_id = self.new_block("for_body".to_string());
                let exit_id = self.new_block("for_exit".to_string());

                // Allocate loop variable
                let elem_ty = match &iter_val.ty {
                    Type::Array(inner, _) | Type::Slice(inner) => (**inner).clone(),
                    _ => Type::Int,
                };
                let reg = self.next_reg();
                let loop_var = self.temp_val(reg, elem_ty.clone());
                self.emit(Instruction::Alloca { dest: loop_var.clone(), ty: elem_ty.clone() });
                let loop_var_slot = loop_var.clone();
                var_map.insert(var.clone(), loop_var);

                // Index variable (allocated on stack, accessed via Pointer)
                let idx_reg = self.next_reg();
                let idx_val = self.temp_val(idx_reg, Type::Int);
                self.emit(Instruction::Alloca { dest: idx_val.clone(), ty: Type::Int });
                self.emit(Instruction::Store {
                    dest: Address::Pointer { val: idx_val.clone(), pointed_to_ty: Type::Int },
                    val: self.int_val(0),
                });

                // Determine array length for bounds check
                let array_len = match &iter_val.ty {
                    Type::Array(_, len) => self.int_val(*len as i64),
                    _ => self.int_val(i64::MAX), // slice: skip bounds check (no runtime length in IR)
                };

                self.set_terminator(Terminator::Jump(header_id));

                // Header: load index and check bounds
                self.switch_to_block(header_id);
                let load_reg = self.next_reg();
                let idx_loaded = self.temp_val(load_reg, Type::Int);
                self.emit(Instruction::Load {
                    dest: idx_loaded.clone(),
                    src: Address::Pointer { val: idx_val.clone(), pointed_to_ty: Type::Int },
                });
                // Bounds check: idx < array_len
                let cmp_reg = self.next_reg();
                let cmp_val = self.temp_val(cmp_reg, Type::Bool);
                self.emit(Instruction::BinaryOperation {
                    dest: cmp_val.clone(),
                    op: BinaryOp::Lt,
                    op1: idx_loaded,
                    op2: array_len,
                });
                self.set_terminator(Terminator::ConditionalJump {
                    cond: cmp_val,
                    then_block: body_id,
                    else_block: exit_id,
                });

                // Body
                self.switch_to_block(body_id);
                // Reload index for this block
                let idx_reload_reg = self.next_reg();
                let idx_reloaded = self.temp_val(idx_reload_reg, Type::Int);
                self.emit(Instruction::Load {
                    dest: idx_reloaded.clone(),
                    src: Address::Pointer { val: idx_val.clone(), pointed_to_ty: Type::Int },
                });
                // Load element from iter at index
                let elem_ptr_reg = self.next_reg();
                let elem_ptr = self.temp_val(elem_ptr_reg, Type::Pointer(Box::new(elem_ty.clone())));
                self.emit(Instruction::GetElementPtr {
                    dest: elem_ptr.clone(),
                    base_ptr: iter_val.clone(),
                    index: idx_reloaded.clone(),
                    element_ty: elem_ty.clone(),
                });
                let loop_var_reg2 = self.next_reg();
                let loop_var2 = self.temp_val(loop_var_reg2, elem_ty.clone());
                self.emit(Instruction::Load {
                    dest: loop_var2.clone(),
                    src: Address::Pointer { val: elem_ptr, pointed_to_ty: elem_ty.clone() },
                });
                // Store element into loop variable slot
                self.emit(Instruction::Store {
                    dest: Address::Pointer { val: loop_var_slot.clone(), pointed_to_ty: elem_ty.clone() },
                    val: loop_var2,
                });
                self.lower_block(body, var_map, return_type);
                // Increment index
                let new_idx_reg = self.next_reg();
                let new_idx = self.temp_val(new_idx_reg, Type::Int);
                self.emit(Instruction::BinaryOperation {
                    dest: new_idx.clone(),
                    op: BinaryOp::Add,
                    op1: idx_reloaded,
                    op2: self.int_val(1),
                });
                self.emit(Instruction::Store {
                    dest: Address::Pointer { val: idx_val.clone(), pointed_to_ty: Type::Int },
                    val: new_idx,
                });
                self.set_terminator(Terminator::Jump(header_id));

                // Exit
                self.switch_to_block(exit_id);
            }
        }
    }

    fn lower_expression(
        &mut self,
        expr: &TypedExpression,
        var_map: &HashMap<String, TypedValue>,
    ) -> TypedValue {
        match &expr.node {
            TypedExpressionKind::FloatLiteral(f) => self.float_val(*f),
            TypedExpressionKind::IntLiteral(n) => self.int_val(*n),
            TypedExpressionKind::BoolLiteral(b) => self.bool_val(*b),
            TypedExpressionKind::StringLiteral(s) => {
                let name = self.next_string_name();
                self.global_strings.push(IrGlobalString { name: name.clone(), value: s.clone() });
                self.string_val(name)
            }
            TypedExpressionKind::Variable(name) => {
                if let Some(slot) = var_map.get(name) {
                    let reg = self.next_reg();
                    let dest = self.temp_val(reg, slot.ty.clone());
                    self.emit(Instruction::Load {
                        dest: dest.clone(),
                        src: Address::Pointer { val: slot.clone(), pointed_to_ty: slot.ty.clone() },
                    });
                    dest
                } else {
                    self.var_val(name, expr.ty.clone())
                }
            }
            TypedExpressionKind::FunctionCall { name, args } => {
                let lowered_args: Vec<TypedValue> = args.iter()
                    .map(|a| self.lower_expression(a, var_map))
                    .collect();
                let dest = if expr.ty != Type::Void {
                    let reg = self.next_reg();
                    Some(self.temp_val(reg, expr.ty.clone()))
                } else {
                    None
                };
                self.emit(Instruction::Call {
                    dest: dest.clone(),
                    func_name: name.clone(),
                    args: lowered_args,
                });
                dest.unwrap_or_else(|| self.temp_val(0, Type::Void))
            }
            TypedExpressionKind::BinaryOperation { left, right, op } => {
                let l = self.lower_expression(left, var_map);
                let r = self.lower_expression(right, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::BinaryOperation {
                    dest: dest.clone(),
                    op: op.clone(),
                    op1: l,
                    op2: r,
                });
                dest
            }
            TypedExpressionKind::UnaryOperation { op, expr: inner } => {
                let operand = self.lower_expression(inner, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                match op {
                    crate::ast::UnaryOp::Not => {
                        self.emit(Instruction::UnaryNot { dest: dest.clone(), operand });
                    }
                    crate::ast::UnaryOp::Neg => {
                        self.emit(Instruction::BinaryOperation {
                            dest: dest.clone(),
                            op: BinaryOp::Sub,
                            op1: self.int_val(0),
                            op2: operand,
                        });
                    }
                }
                dest
            }
            TypedExpressionKind::ArrayLiteral(elems) => {
                let lowered: Vec<TypedValue> = elems.iter()
                    .map(|e| self.lower_expression(e, var_map))
                    .collect();
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::Alloca { dest: dest.clone(), ty: expr.ty.clone() });
                for (i, elem) in lowered.iter().enumerate() {
                    let idx = self.int_val(i as i64);
                    let elem_ptr_reg = self.next_reg();
                    let elem_ptr = self.temp_val(elem_ptr_reg, Type::Pointer(Box::new(elem.ty.clone())));
                    self.emit(Instruction::GetElementPtr {
                        dest: elem_ptr.clone(),
                        base_ptr: dest.clone(),
                        index: idx,
                        element_ty: elem.ty.clone(),
                    });
                    self.emit(Instruction::Store {
                        dest: Address::Pointer { val: elem_ptr, pointed_to_ty: elem.ty.clone() },
                        val: elem.clone(),
                    });
                }
                dest
            }
            TypedExpressionKind::StructLiteral { name, fields } => {
                let reg = self.next_reg();
                let dest = self.temp_val(reg, Type::Struct(name.clone()));
                self.emit(Instruction::Alloca { dest: dest.clone(), ty: Type::Struct(name.clone()) });
                for (i, (_, _, field_expr)) in fields.iter().enumerate() {
                    let val = self.lower_expression(field_expr, var_map);
                    let field_ptr_reg = self.next_reg();
                    let field_ptr = self.temp_val(field_ptr_reg, Type::Pointer(Box::new(val.ty.clone())));
                    self.emit(Instruction::GetFieldPtr {
                        dest: field_ptr.clone(),
                        base_ptr: dest.clone(),
                        field_index: i,
                        field_ty: val.ty.clone(),
                        struct_name: name.clone(),
                    });
                    self.emit(Instruction::Store {
                        dest: Address::Pointer { val: field_ptr, pointed_to_ty: val.ty.clone() },
                        val,
                    });
                }
                dest
            }
            TypedExpressionKind::MemberAccess { base, field: _, field_index } => {
                let base_val = self.lower_expression(base, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                if let Type::Struct(name) = &base_val.ty {
                    let field_idx = *field_index;
                    let field_ptr_reg = self.next_reg();
                    let field_ptr = self.temp_val(field_ptr_reg, Type::Pointer(Box::new(expr.ty.clone())));
                    let name_clone = name.clone();
                    self.emit(Instruction::GetFieldPtr {
                        dest: field_ptr.clone(),
                        base_ptr: base_val,
                        field_index: field_idx,
                        field_ty: expr.ty.clone(),
                        struct_name: name_clone,
                    });
                    self.emit(Instruction::Load {
                        dest: dest.clone(),
                        src: Address::Pointer { val: field_ptr, pointed_to_ty: expr.ty.clone() },
                    });
                }
                dest
            }
            TypedExpressionKind::AddressOf(inner) => {
                let inner_val = self.lower_expression(inner, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, Type::Pointer(Box::new(inner_val.ty.clone())));
                self.emit(Instruction::Comment(format!("address_of {}", reg)));
                dest
            }
            TypedExpressionKind::Dereference(inner) => {
                let ptr_val = self.lower_expression(inner, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::Load {
                    dest: dest.clone(),
                    src: Address::Pointer { val: ptr_val, pointed_to_ty: expr.ty.clone() },
                });
                dest
            }
            TypedExpressionKind::Index { array, index } => {
                let arr_val = self.lower_expression(array, var_map);
                let idx_val = self.lower_expression(index, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                let elem_ptr_reg = self.next_reg();
                let elem_ptr = self.temp_val(elem_ptr_reg, Type::Pointer(Box::new(expr.ty.clone())));
                self.emit(Instruction::GetElementPtr {
                    dest: elem_ptr.clone(),
                    base_ptr: arr_val,
                    index: idx_val,
                    element_ty: expr.ty.clone(),
                });
                self.emit(Instruction::Load {
                    dest: dest.clone(),
                    src: Address::Pointer { val: elem_ptr, pointed_to_ty: expr.ty.clone() },
                });
                dest
            }
            TypedExpressionKind::Match { scrutinee, arms } => {
                let scrutinee_val = self.lower_expression(scrutinee, var_map);

                let merge_id = self.new_block("match_merge".to_string());
                let default_id = self.new_block("match_default".to_string());

                let n_arms = arms.len();
                let mut arm_body_blocks: Vec<usize> = Vec::new();
                let mut test_blocks: Vec<usize> = Vec::new();

                // Create arm body blocks
                for i in 0..n_arms {
                    arm_body_blocks.push(self.new_block(format!("match_arm_{}", i)));
                }

                // Create test blocks for all arms except the last
                for i in 0..n_arms.saturating_sub(1) {
                    test_blocks.push(self.new_block(format!("match_test_{}", i)));
                }

                if n_arms == 0 {
                    self.set_terminator(Terminator::Unreachable);
                    self.switch_to_block(default_id);
                    self.set_terminator(Terminator::Unreachable);
                    self.switch_to_block(merge_id);
                    let result_reg = self.next_reg();
                    return self.temp_val(result_reg, expr.ty.clone());
                }

                // Jump to first test block (or first arm body if only 1 arm)
                if test_blocks.is_empty() {
                    self.set_terminator(Terminator::Jump(arm_body_blocks[0]));
                } else {
                    self.set_terminator(Terminator::Jump(test_blocks[0]));
                }

                // Generate test blocks for arms 0..n-1
                for i in 0..n_arms.saturating_sub(1) {
                    let arm = &arms[i];
                    self.switch_to_block(test_blocks[i]);

                    // Determine next block if this test fails
                    let next_block = if i + 1 < test_blocks.len() {
                        test_blocks[i + 1]
                    } else {
                        arm_body_blocks[n_arms - 1] // last arm body is fallthrough
                    };

                    match arm.pattern.kind.as_str() {
                        "wildcard" | "var" => {
                            self.set_terminator(Terminator::Jump(arm_body_blocks[i]));
                        }
                        "int" => {
                            let cmp_reg = self.next_reg();
                            let cmp_val = self.temp_val(cmp_reg, Type::Bool);
                            self.emit(Instruction::BinaryOperation {
                                dest: cmp_val.clone(),
                                op: BinaryOp::Eq,
                                op1: scrutinee_val.clone(),
                                op2: self.int_val(arm.pattern.int_val),
                            });
                            self.set_terminator(Terminator::ConditionalJump {
                                cond: cmp_val,
                                then_block: arm_body_blocks[i],
                                else_block: next_block,
                            });
                        }
                        "bool" => {
                            let cmp_reg = self.next_reg();
                            let cmp_val = self.temp_val(cmp_reg, Type::Bool);
                            self.emit(Instruction::BinaryOperation {
                                dest: cmp_val.clone(),
                                op: BinaryOp::Eq,
                                op1: scrutinee_val.clone(),
                                op2: self.bool_val(arm.pattern.bool_val),
                            });
                            self.set_terminator(Terminator::ConditionalJump {
                                cond: cmp_val,
                                then_block: arm_body_blocks[i],
                                else_block: next_block,
                            });
                        }
                        "string" => {
                            let cmp_reg = self.next_reg();
                            let cmp_val = self.temp_val(cmp_reg, Type::Int);
                            self.emit(Instruction::Call {
                                dest: Some(cmp_val.clone()),
                                func_name: "fusion_strcmp".to_string(),
                                args: vec![
                                    scrutinee_val.clone(),
                                    self.string_val(arm.pattern.str_val.clone()),
                                ],
                            });
                            let eq_reg = self.next_reg();
                            let eq_val = self.temp_val(eq_reg, Type::Bool);
                            self.emit(Instruction::BinaryOperation {
                                dest: eq_val.clone(),
                                op: BinaryOp::Eq,
                                op1: cmp_val,
                                op2: self.int_val(0),
                            });
                            self.set_terminator(Terminator::ConditionalJump {
                                cond: eq_val,
                                then_block: arm_body_blocks[i],
                                else_block: next_block,
                            });
                        }
                        _ => {
                            self.set_terminator(Terminator::Jump(next_block));
                        }
                    }
                }

                // Generate arm body blocks
                for i in 0..n_arms {
                    let arm = &arms[i];
                    self.switch_to_block(arm_body_blocks[i]);

                    // Determine fallback block for guard failure
                    let guard_fail_block = if i + 1 < test_blocks.len() {
                        test_blocks[i + 1]
                    } else if i + 1 < n_arms {
                        arm_body_blocks[i + 1]
                    } else {
                        default_id
                    };

                    // Handle guard
                    if let Some(guard) = &arm.guard {
                        let guard_val = self.lower_expression(guard, var_map);
                        let guard_pass_id = self.new_block(format!("match_guard_pass_{}", i));
                        self.set_terminator(Terminator::ConditionalJump {
                            cond: guard_val,
                            then_block: guard_pass_id,
                            else_block: guard_fail_block,
                        });
                        self.switch_to_block(guard_pass_id);
                    }

                    let val = self.lower_expression(&arm.body, var_map);
                    let reg = self.next_reg();
                    let result = self.temp_val(reg, expr.ty.clone());
                    self.emit(Instruction::Copy { dest: result, src: val });
                    self.set_terminator(Terminator::Jump(merge_id));
                }

                // Default
                self.switch_to_block(default_id);
                self.set_terminator(Terminator::Unreachable);

                // Merge
                self.switch_to_block(merge_id);
                let result_reg = self.next_reg();
                self.temp_val(result_reg, expr.ty.clone())
            }
            TypedExpressionKind::Closure { params: _, body } => {
                let body_val = self.lower_expression(body, var_map);
                let reg = self.next_reg();
                let dest = self.temp_val(reg, expr.ty.clone());
                self.emit(Instruction::MakeClosure {
                    dest: dest.clone(),
                    func_name: String::new(),
                    captured: vec![body_val],
                });
                dest
            }
        }
    }

    fn lower_lvalue(
        &mut self,
        expr: &TypedExpression,
        var_map: &HashMap<String, TypedValue>,
    ) -> Address {
        match &expr.node {
            TypedExpressionKind::Variable(name) => {
                if let Some(var) = var_map.get(name) {
                    Address::Variable { name: name.clone(), ty: var.ty.clone() }
                } else {
                    Address::Variable { name: name.clone(), ty: expr.ty.clone() }
                }
            }
            TypedExpressionKind::MemberAccess { base, field: _, field_index } => {
                let base_addr = self.lower_lvalue(base, var_map);
                let base_val = self.lower_expression(base, var_map);
                if let Type::Struct(struct_name) = &base_val.ty {
                    Address::Field {
                        base: Box::new(base_addr),
                        field_index: *field_index,
                        field_ty: expr.ty.clone(),
                        struct_name: struct_name.clone(),
                    }
                } else {
                    Address::Variable { name: "unknown".to_string(), ty: expr.ty.clone() }
                }
            }
            TypedExpressionKind::Dereference(inner) => {
                let ptr_val = self.lower_expression(inner, var_map);
                Address::Pointer { val: ptr_val, pointed_to_ty: expr.ty.clone() }
            }
            TypedExpressionKind::Index { array, index } => {
                let arr_val = self.lower_expression(array, var_map);
                let idx_val = self.lower_expression(index, var_map);
                Address::Element {
                    base: Box::new(Address::Variable { name: "array".to_string(), ty: arr_val.ty.clone() }),
                    index: idx_val,
                    element_ty: expr.ty.clone(),
                }
            }
            _ => Address::Variable { name: "unknown".to_string(), ty: expr.ty.clone() },
        }
    }
}

/// Lower a typed program to an IR module.
pub fn lower_program(prog: &TypedProgram) -> IrModule {
    let mut builder = IrBuilder::new();
    builder.lower_program(prog)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::Span;

    #[test]
    fn test_lower_empty_program() {
        let prog = TypedProgram {
            functions: Vec::new(),
            structs: Vec::new(),
        };
        let module = lower_program(&prog);
        assert!(module.functions.is_empty());
        assert!(module.structs.is_empty());
    }

    #[test]
    fn test_lower_function_with_return() {
        use crate::ir::Type;
        let func = TypedFunction {
            name: "main".to_string(),
            params: Vec::new(),
            return_type: Type::Int,
            body: vec![
                TypedStatement::Return(Some(TypedExpression {
                    node: TypedExpressionKind::IntLiteral(42),
                    ty: Type::Int,
                    span: Span::default(),
                })),
            ],
        };
        let prog = TypedProgram {
            functions: vec![func],
            structs: Vec::new(),
        };
        let module = lower_program(&prog);
        assert_eq!(module.functions.len(), 1);
        assert_eq!(module.functions[0].name, "main");
        assert!(!module.functions[0].blocks.is_empty());
    }
}