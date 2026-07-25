//! IR Function Inlining Pass
//! Addresses: Missing optimization passes (Inlining).
use crate::types::*;
use std::collections::HashMap;

use crate::ir::{IrModule, IrFunction, BasicBlock, Instruction, TypedValue, Value, Terminator};

pub struct Inliner {
    /// Maximum number of instructions a function can have to be considered for inlining
    threshold: usize,
}

impl Inliner {
    pub fn new() -> Self {
        Self {
            threshold: 20, // Inline small functions (<= 20 instructions)
        }
    }

    pub fn run(&mut self, module: &mut IrModule) {
        let mut inline_candidates = HashMap::new();

        // Step 1: Identify small functions that are safe to inline
        for func in &module.functions {
            let total_instrs: usize = func.blocks.iter().map(|b| b.instrs.len()).sum();
            if total_instrs <= self.threshold && func.name != "main" {
                inline_candidates.insert(func.name.clone(), func.clone());
            }
        }

        // Step 2: Traverse all functions and splice in basic blocks at call sites
        for func in &mut module.functions {
            self.inline_into(func, &inline_candidates);
        }
    }

    fn inline_into(&self, caller: &mut IrFunction, candidates: &FMap<FString, IrFunction>) {
        let mut changed = true;

        while changed {
            changed = false;

            // Find the first inlineable call across all blocks
            let mut splice_info: Option<(usize, usize, FString)> = None; // (block_idx, instr_idx, func_name)
            for (block_idx, block) in caller.blocks.iter().enumerate() {
                for (i, instr) in block.instrs.iter().enumerate() {
                    if let Instruction::Call { func_name, .. } = instr {
                        if candidates.contains_key(func_name) {
                            splice_info = Some((block_idx, i, func_name.clone()));
                            break;
                        }
                    }
                }
                if splice_info.is_some() {
                    break;
                }
            }

            if let Some((block_idx, instr_idx, target_name)) = splice_info {
                let callee = candidates[&target_name].clone();
                self.splice_inline(caller, &callee, block_idx, instr_idx, &target_name);
                changed = true;
            }
        }
    }

    /// Split the caller block at `instr_idx`, inline the callee's body, and rewire CFG edges.
    fn splice_inline(
        &self,
        caller: &mut IrFunction,
        callee: &IrFunction,
        block_idx: usize,
        instr_idx: usize,
        target_name: &str,
    ) {
        // Extract the call instruction to read its dest/args
        let call_instr = caller.blocks[block_idx].instrs.remove(instr_idx);
        let (call_dest, call_args) = match call_instr {
            Instruction::Call { dest, func_name: _, args } => (dest, args),
            _ => unreachable!(),
        };

        // The remainder of the block after the call site becomes the "after" block
        let after_instrs: Vec<Instruction> = caller.blocks[block_idx].instrs.drain(instr_idx..).collect();
        let after_terminator = caller.blocks[block_idx].terminator.clone();

        // Compute register offset to avoid name collisions
        let reg_offset = caller.blocks.iter()
            .flat_map(|b| &b.instrs)
            .filter_map(|i| match i {
                Instruction::BinaryOperation { dest, .. }
                | Instruction::Copy { dest, .. }
                | Instruction::UnaryNot { dest, .. }
                | Instruction::Load { dest, .. }
                | Instruction::Alloca { dest, .. }
                | Instruction::GetElementPtr { dest, .. }
                | Instruction::GetFieldPtr { dest, .. }
                | Instruction::Phi { dest, .. }
                | Instruction::MakeClosure { dest, .. } => Some(dest),
                _ => None,
            })
            .filter_map(|tv| match &tv.val {
                Value::Temp(id) => Some(*id),
                _ => None,
            })
            .max()
            .unwrap_or(0) + 1;

        // Build parameter map: callee param names -> actual argument values
        let mut param_map: FMap<String, TypedValue> = HashMap::new();
        for (i, (param_name, _)) in callee.params.iter().enumerate() {
            if let Some(arg) = call_args.get(i) {
                param_map.insert(param_name.clone(), arg.clone());
            }
        }

        // Clone and remap callee blocks
        let mut inlined_blocks: Vec<BasicBlock> = callee.blocks.iter().map(|b| {
            let mut block = b.clone();
            for instr in &mut block.instrs {
                self.remap_instruction(instr, reg_offset, &param_map);
            }
            block
        }).collect();

        // Remap terminators in inlined blocks
        for block in &mut inlined_blocks {
            self.remap_terminator(&mut block.terminator, reg_offset, &param_map);
        }

        // Wire entry: the caller block jumps into the inlined entry
        let inlined_entry = caller.blocks.len();
        caller.blocks[block_idx].terminator = Terminator::Jump(inlined_entry);

        // The last inlined block (callee exit) should jump to the "after" block
        let after_block_idx = inlined_entry + inlined_blocks.len();

        // Update the callee's last block terminator to jump to after_block
        let last_inlined = inlined_blocks.last_mut().unwrap();
        match &mut last_inlined.terminator {
            Terminator::Return(ret_val) => {
                // If callee returns a value and caller expects one, insert a Copy
                let ret_val = ret_val.take();
                if let (Some(dest), Some(ret)) = (&call_dest, ret_val) {
                    last_inlined.instrs.push(Instruction::Copy {
                        dest: dest.clone(),
                        src: ret,
                    });
                }
                last_inlined.terminator = Terminator::Jump(after_block_idx);
            }
            Terminator::Jump(_) => {
                last_inlined.terminator = Terminator::Jump(after_block_idx);
            }
            Terminator::ConditionalJump { .. } => {
                // Both branches need to point to the after block
                // This is a simplification; proper inlining would need edge splitting
                last_inlined.terminator = Terminator::Jump(after_block_idx);
            }
            Terminator::Unreachable => {
                // Keep unreachable as-is; the after block is dead code anyway
            }
        }

        // Remap inlined block indices to be contiguous in caller
        let base_offset = caller.blocks.len();
        for block in &mut inlined_blocks {
            match &mut block.terminator {
                Terminator::Jump(target) => {
                    if *target < callee.blocks.len() {
                        *target = base_offset + *target;
                    }
                }
                Terminator::ConditionalJump { then_block, else_block, .. } => {
                    if *then_block < callee.blocks.len() {
                        *then_block = base_offset + *then_block;
                    }
                    if *else_block < callee.blocks.len() {
                        *else_block = base_offset + *else_block;
                    }
                }
                _ => {}
            }
        }

        // Append inlined blocks to caller
        caller.blocks.extend(inlined_blocks);

        // Create the "after" block with the remainder of the original block
        caller.blocks.push(BasicBlock {
            label: format!("{}_after_inline_{}", caller.blocks[block_idx].label, target_name),
            instrs: after_instrs,
            terminator: after_terminator,
        });
    }

    fn remap_instruction(
        &self,
        instr: &mut Instruction,
        offset: usize,
        param_map: &FMap<String, TypedValue>,
    ) {
        match instr {
            Instruction::BinaryOperation { dest, op1, op2, .. } => {
                self.remap_typed_value(dest, offset, param_map);
                self.remap_typed_value(op1, offset, param_map);
                self.remap_typed_value(op2, offset, param_map);
            }
            Instruction::Copy { dest, src } => {
                self.remap_typed_value(dest, offset, param_map);
                self.remap_typed_value(src, offset, param_map);
            }
            Instruction::UnaryNot { dest, operand } => {
                self.remap_typed_value(dest, offset, param_map);
                self.remap_typed_value(operand, offset, param_map);
            }
            Instruction::Load { dest, .. } => {
                self.remap_typed_value(dest, offset, param_map);
            }
            Instruction::Call { dest, args, .. } => {
                if let Some(d) = dest {
                    self.remap_typed_value(d, offset, param_map);
                }
                for arg in args {
                    self.remap_typed_value(arg, offset, param_map);
                }
            }
            Instruction::Alloca { dest, .. } => {
                self.remap_typed_value(dest, offset, param_map);
            }
            Instruction::GetElementPtr { dest, base_ptr, index, .. } => {
                self.remap_typed_value(dest, offset, param_map);
                self.remap_typed_value(base_ptr, offset, param_map);
                self.remap_typed_value(index, offset, param_map);
            }
            Instruction::GetFieldPtr { dest, base_ptr, .. } => {
                self.remap_typed_value(dest, offset, param_map);
                self.remap_typed_value(base_ptr, offset, param_map);
            }
            Instruction::Phi { dest, incoming } => {
                self.remap_typed_value(dest, offset, param_map);
                for (val, _) in incoming {
                    self.remap_typed_value(val, offset, param_map);
                }
            }
            Instruction::MakeClosure { dest, captured, .. } => {
                self.remap_typed_value(dest, offset, param_map);
                for arg in captured {
                    self.remap_typed_value(arg, offset, param_map);
                }
            }
            Instruction::Store { val, .. } => {
                self.remap_typed_value(val, offset, param_map);
            }
            _ => {}
        }
    }

    fn remap_terminator(
        &self,
        term: &mut Terminator,
        offset: usize,
        param_map: &FMap<String, TypedValue>,
    ) {
        match term {
            Terminator::ConditionalJump { cond, .. } => {
                self.remap_typed_value(cond, offset, param_map);
            }
            Terminator::Return(Some(val)) => {
                self.remap_typed_value(val, offset, param_map);
            }
            _ => {}
        }
    }

    fn remap_typed_value(&self, tv: &mut TypedValue, offset: usize, param_map: &FMap<String, TypedValue>) {
        // Substitute parameters first
        if let Value::Variable(name) = &tv.val {
            if let Some(arg_val) = param_map.get(name) {
                *tv = arg_val.clone();
                return;
            }
        }
        // Then remap temporaries to avoid collisions
        if let Value::Temp(id) = &mut tv.val {
            *id += offset;
        }
    }
}