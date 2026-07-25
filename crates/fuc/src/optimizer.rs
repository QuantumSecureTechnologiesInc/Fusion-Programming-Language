//! IR optimization passes.
use crate::types::*;
use std::collections::HashMap;
use crate::ast::BinaryOp;
use crate::ir::{Address, BasicBlock, Instruction, IrFunction, IrModule, TypedValue, Value};

/// Applies optimization passes to the module.
pub fn optimize(module: IrModule) -> IrModule {
    let mut module = module;
    for func in &mut module.functions {
        optimize_function(func);
    }
    module
}

fn resolve_address(addr: &Address, consts: &FMap<Value, TypedValue>) -> Address {
    let resolve = |v: &TypedValue, consts: &FMap<Value, TypedValue>| -> TypedValue {
        consts.get(&v.val).cloned().unwrap_or_else(|| v.clone())
    };
    match addr {
        Address::Variable { name, ty } => {
            Address::Variable {
                name: name.clone(),
                ty: ty.clone(),
            }
        }
        Address::Pointer { val, pointed_to_ty } => {
            Address::Pointer {
                val: resolve(val, consts),
                pointed_to_ty: pointed_to_ty.clone(),
            }
        }
        Address::Element { base, index, element_ty } => {
            Address::Element {
                base: Box::new(resolve_address(base, consts)),
                index: resolve(index, consts),
                element_ty: element_ty.clone(),
            }
        }
        Address::Field { base, field_index, field_ty, struct_name } => {
            Address::Field {
                base: Box::new(resolve_address(base, consts)),
                field_index: *field_index,
                field_ty: field_ty.clone(),
                struct_name: struct_name.clone(),
            }
        }
    }
}

fn optimize_function(func: &mut IrFunction) {
    let mut changed = true;
    while changed {
        changed = false;
        
        // Pass 1: Constant Folding
        let mut block_indices: FVec<FSize> = Vec::new();
        let mut idx = 0;
        while idx < func.blocks.len() {
            block_indices.push(idx);
            idx = idx + 1;
        }
        for block_idx in block_indices {
            if let Some(block) = func.blocks.get_mut(block_idx) {
                if constant_fold_block(block) {
                    changed = true;
                }
            }
        }

        // Pass 2: Dead Code Elimination (Unreachable Blocks)
        if eliminate_unreachable_blocks(func) {
            changed = true;
        }
        
        // Pass 3: Simplify Control Flow (Jump Threading)
        if simplify_control_flow(func) {
            changed = true;
        }
    }
}

fn constant_fold_block(block: &mut BasicBlock) -> FBool {
    let mut changed = false;
    let mut const_values: FMap<Value, TypedValue> = HashMap::new();
    let resolve = |v: &TypedValue, consts: &FMap<Value, TypedValue>| -> TypedValue {
        consts.get(&v.val).cloned().unwrap_or_else(|| v.clone())
    };
    let mut new_instrs = Vec::new();
    for instr in block.instrs.drain(..) {
        match instr {
            Instruction::BinaryOperation { dest, op, op1, op2 } => {
                let val1 = resolve(&op1, &const_values);
                let val2 = resolve(&op2, &const_values);
                let folded = match (val1.val.clone(), val2.val.clone()) {
                    (Value::IntConst(i1), Value::IntConst(i2)) => {
                        match op {
                            BinaryOp::Add => Some(Value::IntConst(i1 + i2)),
                            BinaryOp::Sub => Some(Value::IntConst(i1 - i2)),
                            BinaryOp::Mul => Some(Value::IntConst(i1 * i2)),
                            BinaryOp::Div => {
                                if i2 != 0 { Some(Value::IntConst(i1 / i2)) } else { None }
                            }
                            BinaryOp::Eq => Some(Value::BoolConst(i1 == i2)),
                            BinaryOp::Neq => Some(Value::BoolConst(i1 != i2)),
                            BinaryOp::Lt => Some(Value::BoolConst(i1 < i2)),
                            BinaryOp::Gt => Some(Value::BoolConst(i1 > i2)),
                            _ => None,
                        }
                    }
                    (Value::BoolConst(b1), Value::BoolConst(b2)) => {
                        match op {
                            BinaryOp::Or => Some(Value::BoolConst(b1 || b2)),
                            BinaryOp::And => Some(Value::BoolConst(b1 && b2)),
                            BinaryOp::Eq => Some(Value::BoolConst(b1 == b2)),
                            BinaryOp::Neq => Some(Value::BoolConst(b1 != b2)),
                            _ => None,
                        }
                    }
                    // Identity: x + 0 -> x
                    (Value::IntConst(0), ref v2) if matches!(op, BinaryOp::Add) => {
                        Some(v2.clone())
                    }
                    (ref v1, Value::IntConst(0)) if matches!(op, BinaryOp::Add) => {
                        Some(v1.clone())
                    }
                    // Identity: x - 0 -> x
                    (ref v1, Value::IntConst(0)) if matches!(op, BinaryOp::Sub) => {
                        Some(v1.clone())
                    }
                    // Identity: x * 1 -> x
                    (Value::IntConst(1), ref v2) if matches!(op, BinaryOp::Mul) => {
                        Some(v2.clone())
                    }
                    (ref v1, Value::IntConst(1)) if matches!(op, BinaryOp::Mul) => {
                        Some(v1.clone())
                    }
                    // Annihilator: x * 0 -> 0
                    (Value::IntConst(0), _) if matches!(op, BinaryOp::Mul) => {
                        Some(Value::IntConst(0))
                    }
                    (_, Value::IntConst(0)) if matches!(op, BinaryOp::Mul) => {
                        Some(Value::IntConst(0))
                    }
                    _ => None,
                };
                if let Some(v) = folded {
                    const_values
                        .insert(
                            dest.val.clone(),
                            TypedValue {
                                val: v,
                                ty: dest.ty.clone(),
                            },
                        );
                    changed = true;
                    continue;
                }
                new_instrs
                    .push(Instruction::BinaryOperation {
                        dest,
                        op,
                        op1: val1,
                        op2: val2,
                    });
            }
            Instruction::Call { dest, func_name, args } => {
                let resolved_args = args
                    .into_iter()
                    .map(|a| resolve(&a, &const_values))
                    .collect();
                new_instrs
                    .push(Instruction::Call {
                        dest,
                        func_name,
                        args: resolved_args,
                    });
            }
            Instruction::Load { dest, src } => {
                let resolved_src = resolve_address(&src, &const_values);
                new_instrs
                    .push(Instruction::Load {
                        dest,
                        src: resolved_src,
                    });
            }
            Instruction::Store { dest, val } => {
                let resolved_dest = resolve_address(&dest, &const_values);
                let resolved_val = resolve(&val, &const_values);
                new_instrs
                    .push(Instruction::Store {
                        dest: resolved_dest,
                        val: resolved_val,
                    });
            }
            Instruction::GetElementPtr { dest, base_ptr, index, element_ty } => {
                let resolved_base = resolve(&base_ptr, &const_values);
                let resolved_index = resolve(&index, &const_values);
                new_instrs
                    .push(Instruction::GetElementPtr {
                        dest,
                        base_ptr: resolved_base,
                        index: resolved_index,
                        element_ty,
                    });
            }
            Instruction::GetFieldPtr {
                dest,
                base_ptr,
                field_index,
                field_ty,
                struct_name,
            } => {
                let resolved_base = resolve(&base_ptr, &const_values);
                new_instrs
                    .push(Instruction::GetFieldPtr {
                        dest,
                        base_ptr: resolved_base,
                        field_index,
                        field_ty,
                        struct_name,
                    });
            }
            Instruction::UnaryNot { dest, operand } => {
                let resolved = resolve(&operand, &const_values);
                if let Value::BoolConst(b) = resolved.val {
                    // !true -> false, !false -> true
                    const_values.insert(
                        dest.val.clone(),
                        TypedValue {
                            val: Value::BoolConst(!b),
                            ty: dest.ty.clone(),
                        },
                    );
                    changed = true;
                    continue;
                }
                new_instrs.push(Instruction::UnaryNot {
                    dest,
                    operand: resolved,
                });
            }
            other => new_instrs.push(other),
        }
    }
    block.instrs = new_instrs;
    
    // Constant fold branch conditions
    match &mut block.terminator {
        crate::ir::Terminator::Return(Some(ref mut v)) => *v = resolve(v, &const_values),
        crate::ir::Terminator::ConditionalJump { ref mut cond, ref mut then_block, ref mut else_block } => {
            *cond = resolve(cond, &const_values);
            if let Value::BoolConst(b) = cond.val {
                // The condition is known statically, convert to unconditional jump
                let target = if b { *then_block } else { *else_block };
                block.terminator = crate::ir::Terminator::Jump(target);
                changed = true;
            }
        }
        _ => {}
    }
    changed
}

/// Removes blocks that cannot be reached from the entry block (DCE).
fn eliminate_unreachable_blocks(func: &mut IrFunction) -> FBool {
    let mut reachable = FSet::new();
    let mut worklist = Vec::new();
    
    reachable.insert(func.entry_block);
    worklist.push(func.entry_block);
    
    while let Some(block_idx) = worklist.pop() {
        let block = &func.blocks[block_idx];
        match &block.terminator {
            crate::ir::Terminator::Jump(target) => {
                if !reachable.contains(target) {
                    reachable.insert(*target);
                    worklist.push(*target);
                }
            }
            crate::ir::Terminator::ConditionalJump { then_block, else_block, .. } => {
                if !reachable.contains(then_block) {
                    reachable.insert(*then_block);
                    worklist.push(*then_block);
                }
                if !reachable.contains(else_block) {
                    reachable.insert(*else_block);
                    worklist.push(*else_block);
                }
            }
            _ => {}
        }
    }
    
    let mut changed = false;
    for i in 0..func.blocks.len() {
        if !reachable.contains(&i) && func.blocks[i].terminator.tag() != crate::ir::TERM_RETURN {
            // Replace terminator with a dummy return so codegen doesn't crash on disconnected blocks
            func.blocks[i].instrs.clear();
            func.blocks[i].terminator = crate::ir::return_terminator(None);
            changed = true;
        }
    }
    
    changed
}

/// Removes trivial jumps to blocks that only contain a jump.
fn simplify_control_flow(func: &mut IrFunction) -> FBool {
    let mut changed = false;
    let mut replacements = HashMap::new();
    
    for (i, block) in func.blocks.iter().enumerate() {
        if block.instrs.is_empty() {
            if let crate::ir::Terminator::Jump(target) = &block.terminator {
                if i != *target && i != func.entry_block {
                    replacements.insert(i, *target);
                }
            }
        }
    }
    
    if replacements.is_empty() {
        return false;
    }
    
    for block in &mut func.blocks {
        match &mut block.terminator {
            crate::ir::Terminator::Jump(target) => {
                if let Some(new_target) = replacements.get(target) {
                    *target = *new_target;
                    changed = true;
                }
            }
            crate::ir::Terminator::ConditionalJump { then_block, else_block, .. } => {
                if let Some(new_target) = replacements.get(then_block) {
                    *then_block = *new_target;
                    changed = true;
                }
                if let Some(new_target) = replacements.get(else_block) {
                    *else_block = *new_target;
                    changed = true;
                }
            }
            _ => {}
        }
    }
    
    changed
}