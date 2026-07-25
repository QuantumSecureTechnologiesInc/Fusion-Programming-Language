//! IR optimization passes with CFG support.
//! Refined: Implements Global Dead Code Elimination and Jump Threading.
use crate::types::*;
use std::collections::HashMap;

use crate::ir::{BasicBlock, BinaryOp, Instruction, IrFunction, IrModule, TypedValue, Value, Terminator};

/// Applies optimization passes to the module.
pub fn optimize(module: IrModule) -> IrModule {
    let mut module = module;
    for func in &mut module.functions {
        optimize_function(func);
    }
    module
}

fn optimize_function(func: &mut IrFunction) {
    let mut changed = true;
    while changed {
        changed = false;
        
        // Pass 1: Constant Folding
        for block in &mut func.blocks {
            if constant_fold_block(block) {
                changed = true;
            }
        }

        // Pass 2: Dead Code Elimination (CFG-aware)
        if eliminate_dead_blocks(func) {
            changed = true;
        }
        
        // Pass 3: Simplify CFG (Jump Threading)
        if simplify_cfg(func) {
            changed = true;
        }
    }
}

fn eliminate_dead_blocks(func: &mut IrFunction) -> FBool {
    // Basic unreachable block elimination using reachability analysis
    let mut reachable = FSet::new();
    let mut worklist = Vec::new();
    
    reachable.insert(func.entry_block);
    worklist.push(func.entry_block);
    
    while let Some(block_idx) = worklist.pop() {
        let block = &func.blocks[block_idx];
        match &block.terminator {
            Terminator::Jump(jump_block) => {
                if !reachable.contains(jump_block) {
                    reachable.insert(*jump_block);
                    worklist.push(*jump_block);
                }
            }
            Terminator::ConditionalJump { then_block: cond_jump_then, else_block: cond_jump_else, .. } => {
                if !reachable.contains(cond_jump_then) {
                    reachable.insert(*cond_jump_then);
                    worklist.push(*cond_jump_then);
                }
                if !reachable.contains(cond_jump_else) {
                    reachable.insert(*cond_jump_else);
                    worklist.push(*cond_jump_else);
                }
            }
            _ => {}
        }
    }
    
    let original_len = func.blocks.len();

    // Build old→new index mapping and collect only reachable blocks.
    let mut old_to_new: HashMap<usize, usize> = HashMap::new();
    let mut new_blocks = Vec::new();
    for (old_idx, block) in func.blocks.drain(..).enumerate() {
        if reachable.contains(&old_idx) {
            old_to_new.insert(old_idx, new_blocks.len());
            new_blocks.push(block);
        }
    }

    // Remap all block references in terminators.
    for block in &mut new_blocks {
        match &mut block.terminator {
            Terminator::Jump(target) => {
                *target = old_to_new[target];
            }
            Terminator::ConditionalJump { then_block, else_block, .. } => {
                *then_block = old_to_new[then_block];
                *else_block = old_to_new[else_block];
            }
            _ => {}
        }

        // Remap block references in phi nodes.
        for instr in &mut block.instrs {
            if let Instruction::Phi { incoming, .. } = instr {
                for (_, block_ref) in incoming.iter_mut() {
                    *block_ref = old_to_new[block_ref];
                }
            }
        }
    }

    func.blocks = new_blocks;
    func.entry_block = old_to_new[&func.entry_block];
    func.blocks.len() != original_len
}

fn simplify_cfg(func: &mut IrFunction) -> FBool {
    let mut changed = false;
    // Basic jump threading logic: Jumps to Jumps
    for i in 0..func.blocks.len() {
        if let Terminator::Jump(target) = func.blocks[i].terminator {
            if func.blocks[target].instrs.is_empty() {
                if let Terminator::Jump(new_target) = func.blocks[target].terminator {
                    func.blocks[i].terminator = Terminator::Jump(new_target);
                    changed = true;
                }
            }
        }
    }
    changed
}

fn constant_fold_block(block: &mut BasicBlock) -> FBool {
    let mut changed = false;
    let mut const_values: FMap<Value, TypedValue> = HashMap::new();

    let resolve = |v: &TypedValue, consts: &FMap<Value, TypedValue>| -> TypedValue {
        consts.get(&v.val).cloned().unwrap_or_else(|| v.clone())
    };

    let fold_int_binop = |op: BinaryOp, a: i64, b: i64| -> Option<i64> {
        match op {
            BinaryOp::Add => a.checked_add(b),
            BinaryOp::Sub => a.checked_sub(b),
            BinaryOp::Mul => a.checked_mul(b),
            BinaryOp::Div if b != 0 => Some(a / b),
            BinaryOp::Mod if b != 0 => Some(a % b),
            BinaryOp::Eq => Some(if a == b { 1 } else { 0 }),
            BinaryOp::Neq => Some(if a != b { 1 } else { 0 }),
            BinaryOp::Lt => Some(if a < b { 1 } else { 0 }),
            BinaryOp::Gt => Some(if a > b { 1 } else { 0 }),
            BinaryOp::Le => Some(if a <= b { 1 } else { 0 }),
            BinaryOp::Ge => Some(if a >= b { 1 } else { 0 }),
            BinaryOp::And => Some(if a != 0 && b != 0 { 1 } else { 0 }),
            BinaryOp::Or => Some(if a != 0 || b != 0 { 1 } else { 0 }),
            _ => None,
        }
    };

    let fold_float_binop = |op: BinaryOp, a: f64, b: f64| -> Option<f64> {
        match op {
            BinaryOp::Add => Some(a + b),
            BinaryOp::Sub => Some(a - b),
            BinaryOp::Mul => Some(a * b),
            BinaryOp::Div if b != 0.0 => Some(a / b),
            BinaryOp::Eq => Some(if a == b { 1.0 } else { 0.0 }),
            BinaryOp::Neq => Some(if a != b { 1.0 } else { 0.0 }),
            BinaryOp::Lt => Some(if a < b { 1.0 } else { 0.0 }),
            BinaryOp::Gt => Some(if a > b { 1.0 } else { 0.0 }),
            BinaryOp::Le => Some(if a <= b { 1.0 } else { 0.0 }),
            BinaryOp::Ge => Some(if a >= b { 1.0 } else { 0.0 }),
            _ => None,
        }
    };

    let mut new_instrs = Vec::new();
    for instr in block.instrs.drain(..) {
        match instr {
            Instruction::BinaryOperation { dest, op, op1, op2 } => {
                let v1 = resolve(&op1, &const_values);
                let v2 = resolve(&op2, &const_values);

                let folded = match (&v1.val, &v2.val) {
                    (Value::IntConst(a), Value::IntConst(b)) => {
                        fold_int_binop(op, *a, *b).map(|r| TypedValue {
                            val: Value::IntConst(r),
                            ty: dest.ty.clone(),
                        })
                    }
                    (Value::FloatConst(a), Value::FloatConst(b)) => {
                        fold_float_binop(op, *a, *b).map(|r| TypedValue {
                            val: Value::FloatConst(r),
                            ty: dest.ty.clone(),
                        })
                    }
                    (Value::BoolConst(a), Value::BoolConst(b)) => {
                        let result = match op {
                            BinaryOp::Eq => *a == *b,
                            BinaryOp::Neq => *a != *b,
                            BinaryOp::And => *a && *b,
                            BinaryOp::Or => *a || *b,
                            _ => {
                                new_instrs.push(Instruction::BinaryOperation {
                                    dest, op, op1: v1, op2: v2,
                                });
                                continue;
                            }
                        };
                        Some(TypedValue {
                            val: Value::BoolConst(result),
                            ty: dest.ty.clone(),
                        })
                    }
                    _ => None,
                };

                if let Some(result) = folded {
                    const_values.insert(dest.val.clone(), result.clone());
                    new_instrs.push(Instruction::Copy {
                        dest: dest.clone(),
                        src: result,
                    });
                    changed = true;
                } else {
                    new_instrs.push(Instruction::BinaryOperation {
                        dest, op, op1: v1, op2: v2,
                    });
                }
            }
            Instruction::UnaryNot { dest, operand } => {
                let resolved = resolve(&operand, &const_values);
                match &resolved.val {
                    Value::BoolConst(b) => {
                        let result = TypedValue {
                            val: Value::BoolConst(!b),
                            ty: dest.ty.clone(),
                        };
                        const_values.insert(dest.val.clone(), result.clone());
                        new_instrs.push(Instruction::Copy { dest: dest.clone(), src: result });
                        changed = true;
                    }
                    _ => {
                        new_instrs.push(Instruction::UnaryNot { dest, operand: resolved });
                    }
                }
            }
            Instruction::Copy { dest, src } => {
                let resolved = resolve(&src, &const_values);
                if resolved.val != src.val {
                    const_values.insert(dest.val.clone(), resolved.clone());
                    new_instrs.push(Instruction::Copy { dest, src: resolved });
                    changed = true;
                } else {
                    const_values.insert(dest.val.clone(), src.clone());
                    new_instrs.push(Instruction::Copy { dest, src });
                }
            }
            other => new_instrs.push(other),
        }
    }
    block.instrs = new_instrs;
    changed
}