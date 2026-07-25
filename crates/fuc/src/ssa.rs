//! Static Single Assignment (SSA) Form construction.
//! Addresses: No SSA form, missing advanced optimizations (GVN, Inlining).
use crate::types::*;
use std::collections::HashMap;

use crate::ir::{self, Instruction, IrFunction, TypedValue, Value, Address, Type};

/// Extended instruction set including SSA Phi nodes
#[derive(Clone, Debug)]
pub enum SsaInstruction {
    Standard(Instruction),
    /// Phi node: selects a value based on the incoming block path
    Phi {
        dest: TypedValue,
        incoming: FVec<(ir::BlockId, TypedValue)>,
    },
}

pub struct SsaBlock {
    pub label: FString,
    pub instrs: FVec<SsaInstruction>,
    pub terminator: ir::Terminator,
    pub predecessors: FVec<ir::BlockId>,
    pub successors: FVec<ir::BlockId>,
}

pub struct SsaFunction {
    pub name: FString,
    pub blocks: FMap<ir::BlockId, SsaBlock>,
    pub entry_block: ir::BlockId,
}

pub struct SsaConverter {
    idom: FMap<ir::BlockId, ir::BlockId>,
    dom_tree_children: FMap<ir::BlockId, FVec<ir::BlockId>>,
    #[allow(dead_code)]
    next_version: FMap<FString, FSize>,
}

impl SsaConverter {
    pub fn new() -> Self {
        Self {
            idom: HashMap::new(),
            dom_tree_children: HashMap::new(),
            next_version: HashMap::new(),
        }
    }

    /// Converts standard linear IR into SSA form
    pub fn convert_function(&mut self, func: &IrFunction) -> SsaFunction {
        let mut ssa_blocks = HashMap::new();

        // Step 1: Build basic CFG (Predecessors and Successors)
        for (i, block) in func.blocks.iter().enumerate() {
            let mut successors = Vec::new();
            match &block.terminator {
                ir::Terminator::Jump(target) => successors.push(*target),
                ir::Terminator::ConditionalJump { then_block, else_block, .. } => {
                    successors.push(*then_block);
                    successors.push(*else_block);
                }
                _ => {}
            }

            let ssa_block = SsaBlock {
                label: block.label.clone(),
                instrs: block.instrs.iter().map(|inst| SsaInstruction::Standard(inst.clone())).collect(),
                terminator: block.terminator.clone(),
                predecessors: Vec::new(), // Filled in Step 2
                successors,
            };
            ssa_blocks.insert(i, ssa_block);
        }

        // Step 2: Backfill predecessors
        let block_ids: FVec<ir::BlockId> = ssa_blocks.keys().cloned().collect();
        for &id in &block_ids {
            let successors = ssa_blocks.get(&id).unwrap().successors.clone();
            for succ_id in successors {
                if let Some(succ_block) = ssa_blocks.get_mut(&succ_id) {
                    succ_block.predecessors.push(id);
                }
            }
        }

        // Step 3: Insert Phi Nodes using Cytron's dominance frontier algorithm
        self.insert_phi_nodes(&mut ssa_blocks);

        // Step 4: Rename variables to use subscripts (x_1, x_2)
        self.rename_variables(&mut ssa_blocks, func.entry_block, &func.params);

        SsaFunction {
            name: func.name.clone(),
            blocks: ssa_blocks,
            entry_block: func.entry_block,
        }
    }

    /// Cytron's algorithm for SSA phi placement:
    /// 1. Compute dominator tree via iterative dataflow
    /// 2. Compute dominance frontiers
    /// 3. Place phi nodes at iterated dominance frontiers of each variable's definitions
    fn insert_phi_nodes(&mut self, blocks: &mut FMap<ir::BlockId, SsaBlock>) {
        let n = blocks.len();
        if n == 0 {
            return;
        }

        let entry = 0;

        let mut block_ids: Vec<ir::BlockId> = blocks.keys().cloned().collect();
        block_ids.sort();

        // --- Phase 1: Compute dominator tree (iterative algorithm) ---
        let mut idom: Vec<Option<ir::BlockId>> = vec![None; n];
        idom[entry] = Some(entry);

        let mut changed = true;
        while changed {
            changed = false;
            for &b in &block_ids {
                if b == entry {
                    continue;
                }
                let block = &blocks[&b];
                let preds: Vec<ir::BlockId> = block.predecessors.iter()
                    .copied()
                    .filter(|&p| p < n && idom[p].is_some())
                    .collect();
                if preds.is_empty() {
                    continue;
                }

                let mut new_idom = preds[0];
                for &p in &preds[1..] {
                    let mut finger1 = new_idom;
                    let mut finger2 = p;
                    while finger1 != finger2 {
                        while finger1 > finger2 {
                            finger1 = idom[finger1].unwrap();
                        }
                        while finger2 > finger1 {
                            finger2 = idom[finger2].unwrap();
                        }
                    }
                    new_idom = finger1;
                }
                if idom[b] != Some(new_idom) {
                    idom[b] = Some(new_idom);
                    changed = true;
                }
            }
        }

        // Store idom for use in rename_variables
        self.idom.clear();
        for (i, &d) in idom.iter().enumerate() {
            if let Some(d) = d {
                self.idom.insert(i, d);
            }
        }

        // --- Phase 2: Compute dominance frontiers ---
        // DF(Y) = { X | X is a predecessor of Y and idom(Y) != X,
        //           OR X is on the path from a predecessor of Y up to idom(Y) }
        let mut df: Vec<FSet<ir::BlockId>> = vec![FSet::new(); n];
        for &y in &block_ids {
            if y == entry {
                continue;
            }
            let idom_y = match idom[y] {
                Some(id) => id,
                None => continue,
            };
            let preds = blocks[&y].predecessors.clone();
            for &p in &preds {
                let mut runner = p;
                while runner != idom_y {
                    df[runner].insert(y);
                    runner = match idom[runner] {
                        Some(id) => id,
                        None => break,
                    };
                }
            }
        }

        // --- Phase 3: Find blocks that define each variable ---
        let mut def_blocks: FMap<String, FSet<ir::BlockId>> = HashMap::new();
        for &b in &block_ids {
            let block = &blocks[&b];
            for instr in &block.instrs {
                match instr {
                    SsaInstruction::Standard(inst) => {
                        if let Some(dest) = get_dest(inst) {
                            if let Value::Variable(name) = &dest.val {
                                def_blocks.entry(name.clone()).or_default().insert(b);
                            }
                        }
                    }
                    SsaInstruction::Phi { dest, .. } => {
                        if let Value::Variable(name) = &dest.val {
                            def_blocks.entry(name.clone()).or_default().insert(b);
                        }
                    }
                }
            }
        }

        // --- Phase 4: Insert phi nodes at iterated dominance frontiers ---
        for (var_name, defs) in &def_blocks {
            let mut phi_inserted: FSet<ir::BlockId> = FSet::new();
            let mut worklist: Vec<ir::BlockId> = defs.iter().copied().collect();
            while let Some(b) = worklist.pop() {
                for &y in &df[b] {
                    if !phi_inserted.contains(&y) {
                        phi_inserted.insert(y);
                        let block = blocks.get_mut(&y).unwrap();
                        let dest = TypedValue {
                            val: Value::Variable(var_name.clone()),
                            ty: Type::Unknown,
                        };
                        let incoming: Vec<(ir::BlockId, TypedValue)> = block.predecessors.iter()
                            .map(|&p| {
                                (p, TypedValue {
                                    val: Value::Variable(var_name.clone()),
                                    ty: Type::Unknown,
                                })
                            })
                            .collect();
                        block.instrs.insert(0, SsaInstruction::Phi {
                            dest,
                            incoming: incoming.into(),
                        });
                        worklist.push(y);
                    }
                }
            }
        }
    }

    /// Renames variables via pre-order dominator tree traversal.
    /// Each definition gets a fresh version number; each use is replaced
    /// with the most recent version on the stack.
    fn rename_variables(
        &mut self,
        blocks: &mut FMap<ir::BlockId, SsaBlock>,
        entry: ir::BlockId,
        params: &[(String, Type)],
    ) {
        // Build dominator tree children map
        self.dom_tree_children.clear();
        for (&child, &parent) in &self.idom {
            if child != parent {
                self.dom_tree_children.entry(parent).or_default().push(child);
            }
        }

        let mut version_stack: FMap<String, Vec<usize>> = HashMap::new();
        let mut version_counter: FMap<String, usize> = HashMap::new();

        // Initialize function parameters with version 1
        for (param_name, _) in params {
            let count = version_counter.entry(param_name.clone()).or_insert(0);
            *count += 1;
            version_stack.entry(param_name.clone()).or_default().push(*count);
        }

        self.rename_block(entry, blocks, &mut version_stack, &mut version_counter);
    }

    fn rename_block(
        &self,
        b: ir::BlockId,
        blocks: &mut FMap<ir::BlockId, SsaBlock>,
        version_stack: &mut FMap<String, Vec<usize>>,
        version_counter: &mut FMap<String, usize>,
    ) {
        let mut defined_vars: Vec<String> = Vec::new();

        // Step 1: Push phi node definitions (phi dests are defined at block entry)
        {
            let block = blocks.get_mut(&b).unwrap();
            for instr in &mut block.instrs {
                if let SsaInstruction::Phi { dest, .. } = instr {
                    let name = match &dest.val {
                        Value::Variable(n) => n.clone(),
                        _ => continue,
                    };
                    let count = version_counter.entry(name.clone()).or_insert(0);
                    *count += 1;
                    version_stack.entry(name.clone()).or_default().push(*count);
                    dest.val = Value::Variable(format!("{}_{}", name, *count));
                    defined_vars.push(name);
                }
            }
        }

        // Step 2: Rename uses in non-phi instructions, then push their defs
        {
            let block = blocks.get_mut(&b).unwrap();
            for instr in &mut block.instrs {
                if let SsaInstruction::Standard(inst) = instr {
                    rename_uses(inst, version_stack);
                    if let Some(dest) = get_dest_mut(inst) {
                        let name = match &dest.val {
                            Value::Variable(n) => n.clone(),
                            _ => continue,
                        };
                        let count = version_counter.entry(name.clone()).or_insert(0);
                        *count += 1;
                        version_stack.entry(name.clone()).or_default().push(*count);
                        dest.val = Value::Variable(format!("{}_{}", name, *count));
                        defined_vars.push(name);
                    }
                }
            }
        }

        // Step 3: Rename uses in terminator
        {
            let block = blocks.get_mut(&b).unwrap();
            match &mut block.terminator {
                ir::Terminator::ConditionalJump { cond, .. } => {
                    rename_tv(cond, version_stack);
                }
                ir::Terminator::Return(Some(v)) => {
                    rename_tv(v, version_stack);
                }
                _ => {}
            }
        }

        // Step 4: Update phi incoming values in successors from this block
        {
            let successors = blocks[&b].successors.clone();
            for s in successors {
                let block = blocks.get_mut(&s).unwrap();
                for instr in &mut block.instrs {
                    if let SsaInstruction::Phi { incoming, .. } = instr {
                        for (pred, val) in incoming.iter_mut() {
                            if *pred == b {
                                rename_tv(val, version_stack);
                            }
                        }
                    }
                }
            }
        }

        // Step 5: Recurse into dominator tree children
        if let Some(children) = self.dom_tree_children.get(&b) {
            let children_clone = children.clone();
            for child in children_clone {
                self.rename_block(child, blocks, version_stack, version_counter);
            }
        }

        // Step 6: Pop all definitions from this block
        for name in &defined_vars {
            if let Some(stack) = version_stack.get_mut(name) {
                stack.pop();
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: extract the destination TypedValue from an instruction
// ---------------------------------------------------------------------------

fn get_dest(inst: &Instruction) -> Option<&TypedValue> {
    match inst {
        Instruction::BinaryOperation { dest, .. }
        | Instruction::Load { dest, .. }
        | Instruction::UnaryNot { dest, .. }
        | Instruction::Phi { dest, .. }
        | Instruction::Copy { dest, .. }
        | Instruction::Alloca { dest, .. }
        | Instruction::GetElementPtr { dest, .. }
        | Instruction::GetFieldPtr { dest, .. } => Some(dest),
        Instruction::Call { dest, .. } => dest.as_ref(),
        _ => None,
    }
}

fn get_dest_mut(inst: &mut Instruction) -> Option<&mut TypedValue> {
    match inst {
        Instruction::BinaryOperation { dest, .. }
        | Instruction::Load { dest, .. }
        | Instruction::UnaryNot { dest, .. }
        | Instruction::Phi { dest, .. }
        | Instruction::Copy { dest, .. }
        | Instruction::Alloca { dest, .. }
        | Instruction::GetElementPtr { dest, .. }
        | Instruction::GetFieldPtr { dest, .. } => Some(dest),
        Instruction::Call { dest, .. } => dest.as_mut(),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Helper: rename all variable *uses* inside an instruction
// ---------------------------------------------------------------------------

fn rename_uses(inst: &mut Instruction, vs: &FMap<String, Vec<usize>>) {
    match inst {
        Instruction::BinaryOperation { op1, op2, .. } => {
            rename_tv(op1, vs);
            rename_tv(op2, vs);
        }
        Instruction::Call { args, .. } => {
            for arg in args.iter_mut() {
                rename_tv(arg, vs);
            }
        }
        Instruction::Load { src, .. } => {
            rename_addr(src, vs);
        }
        Instruction::Store { dest, val } => {
            rename_addr(dest, vs);
            rename_tv(val, vs);
        }
        Instruction::GetElementPtr { base_ptr, index, .. } => {
            rename_tv(base_ptr, vs);
            rename_tv(index, vs);
        }
        Instruction::GetFieldPtr { base_ptr, .. } => {
            rename_tv(base_ptr, vs);
        }
        Instruction::UnaryNot { operand, .. } => {
            rename_tv(operand, vs);
        }
        Instruction::Copy { src, .. } => {
            rename_tv(src, vs);
        }
        Instruction::Phi { incoming, .. } => {
            for (val, _) in incoming.iter_mut() {
                rename_tv(val, vs);
            }
        }
        Instruction::GetAddress { var_name, .. } => {
            let original = var_name.clone();
            if let Some(stack) = vs.get(&original) {
                if let Some(&v) = stack.last() {
                    *var_name = format!("{}_{}", original, v);
                }
            }
        }
        Instruction::MakeClosure { captured, .. } => {
            for val in captured.iter_mut() {
                rename_tv(val, vs);
            }
        }
        _ => {}
    }
}

// ---------------------------------------------------------------------------
// Helper: rename a single TypedValue variable use
// ---------------------------------------------------------------------------

fn rename_tv(tv: &mut TypedValue, vs: &FMap<String, Vec<usize>>) {
    let new_val = match &tv.val {
        Value::Variable(name) => {
            vs.get(name.as_str())
                .and_then(|stack| stack.last())
                .map(|&v| Value::Variable(format!("{}_{}", name, v)))
        }
        _ => None,
    };
    if let Some(v) = new_val {
        tv.val = v;
    }
}

// ---------------------------------------------------------------------------
// Helper: rename variable references inside an Address
// ---------------------------------------------------------------------------

fn rename_addr(addr: &mut Address, vs: &FMap<String, Vec<usize>>) {
    match addr {
        Address::Variable { name, .. } => {
            let original = name.clone();
            if let Some(stack) = vs.get(&original) {
                if let Some(&v) = stack.last() {
                    *name = format!("{}_{}", original, v);
                }
            }
        }
        Address::Pointer { val, .. } => {
            rename_tv(val, vs);
        }
        Address::Element { base, index, .. } => {
            rename_addr(base, vs);
            rename_tv(index, vs);
        }
        Address::Field { base, .. } => {
            rename_addr(base, vs);
        }
    }
}
