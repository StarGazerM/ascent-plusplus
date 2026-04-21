// ddisasm def-use analysis — pure Ascent batch.
// Equivalent rule structure to SRDatalog ddisasm.nim. Uses negation, so
// not compatible with the DD batch-mode backend.
//
// Usage: ddisasm_batch <input_dir> <LOAD> <STORE> <NONE_ACCESS> <PC_RELATIVE>
// The four constants are dataset-specific interned integers (resolved from
// the SRDatalog str2num.json).

use ascent::ascent_par;
use bench_loader::{load_1, load_2, load_3, load_4, load_5, load_6, load_7, load_8};
use std::env;
use std::time::Instant;

type V = i32;

ascent_par! {
    #![measure_rule_times]

    // ===================== EDB relations =====================
    relation arch_memory_access(V, V, V, V, V, V);
    relation arch_reg_reg_arith_op(V, V, V, V, V, V);
    relation arch_return_reg(V);
    relation block_next(V, V, V);
    relation block_last_instr(V, V);
    relation code_in_block(V, V);
    relation direct_call(V, V);
    relation may_fallthrough(V, V);
    relation reg_def_use_block_last_def(V, V, V);
    relation reg_def_use_defined_in_block(V, V);
    relation reg_def_use_flow_def(V, V, V, V);
    relation reg_def_use_live_var_def(V, V, V, V);
    relation reg_def_use_ref_in_block(V, V);
    relation reg_def_use_return_block_end(V, V, V, V);
    relation reg_def_use_used(V, V, V);
    relation reg_def_use_used_in_block(V, V, V, V);
    relation reg_used_for(V, V, V);
    relation rel_jump_table_entry_candidate(V, V, V, V, V, V, V);
    relation stack_def_use_def(V, V, V);
    relation stack_def_use_defined_in_block(V, V, V);
    relation stack_def_use_live_var_def(V, V, V, V, V, V);
    relation stack_def_use_ref_in_block(V, V, V);
    relation stack_def_use_used_in_block(V, V, V, V, V);
    relation stack_def_use_used(V, V, V, V);
    relation stack_def_use_live_var_used_edb(V, V, V, V, V, V, V, V);
    relation jump_table_start(V, V, V, V, V);
    relation def_used_for_address_edb(V, V, V);
    relation stack_def_use_block_last_def(V, V, V, V);

    relation c_load(V);
    relation c_store(V);
    relation c_none_access(V);
    relation c_pc_relative(V);

    // ===================== IDB relations =====================
    relation jump_table_target(V, V);
    relation reg_def_use_def_used(V, V, V, V);
    relation reg_def_use_def_used_proj(V, V, V);
    relation reg_def_use_return_val_used(V, V, V, V, V);
    relation reg_def_use_live_var_used(V, V, V, V);
    relation reg_def_use_live_var_at_prior_used(V, V, V);
    relation reg_def_use_live_var_at_block_end(V, V, V);
    relation reg_reg_arith_op_defs(V, V, V, V, V, V, V, V);
    relation def_used_for_address(V, V, V);
    relation stack_def_use_def_used(V, V, V, V, V, V);
    relation stack_def_use_live_var_at_block_end(V, V, V, V);
    relation stack_def_use_live_var_at_prior_used(V, V, V, V);
    relation stack_def_use_live_var_used_proj(V, V, V, V, V, V);

    // LiveVarUsedProj
    stack_def_use_live_var_used_proj(blk, varr, varp, var_usedr, var_usedp, ea_used) <--
        stack_def_use_live_var_used_edb(blk, varr, varp, var_usedr, var_usedp, ea_used, _, _);

    // JumpTableTargetRule
    jump_table_target(ea, dest) <--
        jump_table_start(ea, size, table_start, _, _),
        rel_jump_table_entry_candidate(_, table_start, size, _, dest, _, _);

    // DefUsedProj
    reg_def_use_def_used_proj(ea_def, mvar, ea_used) <--
        reg_def_use_def_used(ea_def, mvar, ea_used, _);

    // Reg_def_use_def_used rule 1
    reg_def_use_def_used(ea_def, mvar, ea_used, index) <--
        reg_def_use_used(ea_used, mvar, index),
        reg_def_use_block_last_def(ea_used, ea_def, mvar);

    // Rule 2
    reg_def_use_def_used(ea_def, var_identity, ea_used, index) <--
        reg_def_use_live_var_at_block_end(blk, block_used, mvar),
        reg_def_use_live_var_def(blk, var_identity, mvar, ea_def),
        reg_def_use_live_var_used(block_used, mvar, ea_used, index);

    // Rule 3
    reg_def_use_def_used(ea_def, mvar, next_ea_used, next_index) <--
        reg_def_use_live_var_at_prior_used(ea_used, nxt_blk, mvar),
        reg_def_use_def_used_proj(ea_def, mvar, ea_used),
        reg_def_use_live_var_used(nxt_blk, mvar, next_ea_used, next_index);

    // Rule 4
    reg_def_use_def_used(ea_def, reg, ea_used, index) <--
        reg_def_use_return_val_used(_, callee, reg, ea_used, index),
        reg_def_use_return_block_end(callee, _, _, block_end),
        reg_def_use_block_last_def(block_end, ea_def, reg);

    // RetValUsed
    reg_def_use_return_val_used(ea_call, callee, reg, ea_used, index_used) <--
        arch_return_reg(reg),
        reg_def_use_def_used(ea_call, reg, ea_used, index_used),
        direct_call(ea_call, callee);

    // LiveVarUsed 1
    reg_def_use_live_var_used(blk, mvar, ea_used, index) <--
        reg_def_use_used_in_block(blk, ea_used, mvar, index),
        !reg_def_use_block_last_def(ea_used, _, mvar);

    // LiveVarUsed 2
    reg_def_use_live_var_used(ret_block, reg, ea_used, index) <--
        reg_def_use_return_val_used(_, callee, reg, ea_used, index),
        reg_def_use_return_block_end(callee, _, ret_block, ret_block_end),
        !reg_def_use_block_last_def(ret_block_end, _, reg);

    // LiveVarAtPriorUsed
    reg_def_use_live_var_at_prior_used(ea_used, block_used, mvar) <--
        reg_def_use_live_var_at_block_end(blk, block_used, mvar),
        reg_def_use_used_in_block(blk, ea_used, mvar, _),
        !reg_def_use_defined_in_block(blk, mvar);

    // LiveVarAtBlockEnd 1
    reg_def_use_live_var_at_block_end(prev_block, blk, mvar) <--
        block_next(prev_block, prev_block_end, blk),
        reg_def_use_live_var_used(blk, mvar, _, _),
        !reg_def_use_flow_def(prev_block_end, mvar, blk, _);

    // LiveVarAtBlockEnd 2
    reg_def_use_live_var_at_block_end(prev_block, block_used, mvar) <--
        reg_def_use_live_var_at_block_end(blk, block_used, mvar),
        !reg_def_use_ref_in_block(blk, mvar),
        block_next(prev_block, _, blk);

    // RegRegArithDefs
    reg_reg_arith_op_defs(ea, reg_def, ea_def1, reg1, ea_def2, reg2, mult, offset) <--
        def_used_for_address(ea, reg_def, _),
        arch_reg_reg_arith_op(ea, reg_def, reg1, reg2, mult, offset),
        if reg1 != reg2,
        reg_def_use_def_used(ea_def1, reg1, ea, _),
        if ea != ea_def1,
        reg_def_use_def_used(ea_def2, reg2, ea, _),
        if ea != ea_def2;

    // Def_used_for_address rule 1
    def_used_for_address(ea, reg, *pc_rel) <--
        c_pc_relative(pc_rel),
        def_used_for_address_edb(ea, reg, *pc_rel);

    // Rule 2
    def_used_for_address(ea_def, reg, mtype) <--
        reg_def_use_def_used(ea_def, reg, ea, _),
        reg_used_for(ea, reg, mtype);

    // Rule 3
    def_used_for_address(ea_def, reg, mtype) <--
        def_used_for_address(ea_used, _, mtype),
        reg_def_use_def_used(ea_def, reg, ea_used, _);

    // Rule 4
    def_used_for_address(ea_def, reg1, mtype) <--
        c_load(c_ld), c_store(c_st), c_none_access(c_na),
        def_used_for_address(ea_load, reg2, mtype),
        arch_memory_access(*c_ld, ea_load, reg2, reg_base_load, *c_na, stack_pos_load),
        stack_def_use_def_used(ea_store, reg_base_store, stack_pos_store, ea_load, reg_base_load, stack_pos_load),
        arch_memory_access(*c_st, ea_store, reg1, reg_base_store, *c_na, stack_pos_store),
        reg_def_use_def_used(ea_def, reg1, ea_store, _);

    // Stack_def_use_def_used rule 1
    stack_def_use_def_used(ea_def, varr, varp, ea_used, varr, varp) <--
        stack_def_use_used(ea_used, varr, varp, _),
        stack_def_use_block_last_def(ea_used, ea_def, varr, varp);

    // Rule 2
    stack_def_use_def_used(ea_def, def_varr, def_varp, ea_used, var_usedr, var_usedp) <--
        stack_def_use_live_var_at_block_end(blk, block_used, varr, varp),
        stack_def_use_live_var_def(blk, def_varr, def_varp, varr, varp, ea_def),
        stack_def_use_live_var_used_proj(block_used, varr, varp, var_usedr, var_usedp, ea_used);

    // Rule 3
    stack_def_use_def_used(ea_def, def_varr, def_varp, ea_used, used_varr, used_varp) <--
        stack_def_use_live_var_used_proj(ea, def_varr, def_varp, used_varr, used_varp, ea_used),
        may_fallthrough(ea_def, ea),
        code_in_block(ea_def, blk),
        code_in_block(ea, blk),
        stack_def_use_def(ea_def, def_varr, def_varp);

    // Rule 4
    stack_def_use_def_used(ea_def, var_defr, var_defp, next_ea_used, var_usedr, var_usedp) <--
        stack_def_use_live_var_at_prior_used(ea_used, nxt_blk, varr, varp),
        stack_def_use_def_used(ea_def, var_defr, var_defp, ea_used, varr, varp),
        stack_def_use_live_var_used_proj(nxt_blk, varr, varp, var_usedr, var_usedp, next_ea_used);

    // StackLiveVarBlockEnd 1
    stack_def_use_live_var_at_block_end(prev_block, block_used, varr, varp) <--
        stack_def_use_live_var_at_block_end(blk, block_used, varr, varp),
        !stack_def_use_ref_in_block(blk, varr, varp),
        !reg_def_use_defined_in_block(blk, varr),
        block_next(prev_block, _, blk);

    // StackLiveVarBlockEnd 2
    stack_def_use_live_var_at_block_end(prev_block, blk, varr, varp) <--
        block_next(prev_block, _, blk),
        stack_def_use_live_var_used_edb(blk, varr, varp, _, _, _, _, _);

    // StackLiveVarPriorUsed
    stack_def_use_live_var_at_prior_used(ea_used, block_used, varr, varp) <--
        stack_def_use_live_var_at_block_end(blk, block_used, varr, varp),
        stack_def_use_used_in_block(blk, ea_used, varr, varp, _),
        !reg_def_use_defined_in_block(blk, varr),
        !stack_def_use_defined_in_block(blk, varr, varp);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 6 {
        eprintln!("Usage: {} <dir> <LOAD> <STORE> <NONE_ACCESS> <PC_RELATIVE>", args[0]);
        std::process::exit(1);
    }
    let dir = &args[1];
    let p = |i: usize| -> V { args[i].parse().unwrap() };
    let c_load_val = p(2);
    let c_store_val = p(3);
    let c_none_access_val = p(4);
    let c_pc_relative_val = p(5);

    eprintln!("Loading data from {}...", dir);
    eprintln!("  LOAD={}, STORE={}, NONE_ACCESS={}, PC_RELATIVE={}",
              c_load_val, c_store_val, c_none_access_val, c_pc_relative_val);
    let t0 = Instant::now();
    let mut prog = AscentProgram::default();

    macro_rules! load_into {
        ($rel:expr, $data:expr) => { for t in $data { $rel.push(t); } }
    }

    load_into!(prog.arch_memory_access, load_6(dir, "Arch_memory_access_truncate.csv"));
    load_into!(prog.arch_reg_reg_arith_op, load_6(dir, "Arch_reg_reg_arithmetic_operation.csv"));
    load_into!(prog.arch_return_reg, load_1(dir, "Arch_return_reg.csv"));
    load_into!(prog.block_next, load_3(dir, "Block_next.csv"));
    load_into!(prog.block_last_instr, load_2(dir, "Block_last_instruction.csv"));
    load_into!(prog.code_in_block, load_2(dir, "Code_in_block.csv"));
    load_into!(prog.direct_call, load_2(dir, "Direct_call.csv"));
    load_into!(prog.may_fallthrough, load_2(dir, "May_fallthrough.csv"));
    load_into!(prog.reg_def_use_block_last_def, load_3(dir, "Reg_def_use_block_last_def.csv"));
    load_into!(prog.reg_def_use_defined_in_block, load_2(dir, "Reg_def_use_defined_in_block.csv"));
    load_into!(prog.reg_def_use_flow_def, load_4(dir, "Reg_def_use_flow_def.csv"));
    load_into!(prog.reg_def_use_live_var_def, load_4(dir, "Reg_def_use_live_var_def.csv"));
    load_into!(prog.reg_def_use_ref_in_block, load_2(dir, "Reg_def_use_ref_in_block.csv"));
    load_into!(prog.reg_def_use_return_block_end, load_4(dir, "Reg_def_use_return_block_end.csv"));
    load_into!(prog.reg_def_use_used, load_3(dir, "Reg_def_use_used.csv"));
    load_into!(prog.reg_def_use_used_in_block, load_4(dir, "Reg_def_use_used_in_block.csv"));
    load_into!(prog.reg_used_for, load_3(dir, "Reg_used_for.csv"));
    load_into!(prog.rel_jump_table_entry_candidate, load_7(dir, "Relative_jump_table_entry_candidate.csv"));
    load_into!(prog.stack_def_use_def, load_3(dir, "Stack_def_use_def.csv"));
    load_into!(prog.stack_def_use_defined_in_block, load_3(dir, "Stack_def_use_defined_in_block.csv"));
    load_into!(prog.stack_def_use_live_var_def, load_6(dir, "Stack_def_use_live_var_def.csv"));
    load_into!(prog.stack_def_use_ref_in_block, load_3(dir, "Stack_def_use_ref_in_block.csv"));
    load_into!(prog.stack_def_use_used_in_block, load_5(dir, "Stack_def_use_used_in_block.csv"));
    load_into!(prog.stack_def_use_used, load_4(dir, "Stack_def_use_used.csv"));
    load_into!(prog.stack_def_use_live_var_used_edb, load_8(dir, "Stack_def_use_live_var_used.csv"));
    load_into!(prog.jump_table_start, load_5(dir, "Jump_table_start.csv"));
    load_into!(prog.def_used_for_address_edb, load_3(dir, "Def_used_for_address.csv"));
    load_into!(prog.stack_def_use_block_last_def, load_4(dir, "Stack_def_use_block_last_def.csv"));

    prog.c_load.push((c_load_val,));
    prog.c_store.push((c_store_val,));
    prog.c_none_access.push((c_none_access_val,));
    prog.c_pc_relative.push((c_pc_relative_val,));

    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("jump_table_target: {}", prog.jump_table_target.len());
    eprintln!("reg_def_use_def_used: {}", prog.reg_def_use_def_used.len());
    eprintln!("reg_def_use_return_val_used: {}", prog.reg_def_use_return_val_used.len());
    eprintln!("reg_def_use_live_var_used: {}", prog.reg_def_use_live_var_used.len());
    eprintln!("reg_def_use_live_var_at_prior_used: {}", prog.reg_def_use_live_var_at_prior_used.len());
    eprintln!("reg_def_use_live_var_at_block_end: {}", prog.reg_def_use_live_var_at_block_end.len());
    eprintln!("reg_reg_arith_op_defs: {}", prog.reg_reg_arith_op_defs.len());
    eprintln!("def_used_for_address: {}", prog.def_used_for_address.len());
    eprintln!("stack_def_use_def_used: {}", prog.stack_def_use_def_used.len());
    eprintln!("stack_def_use_live_var_at_block_end: {}", prog.stack_def_use_live_var_at_block_end.len());
    eprintln!("stack_def_use_live_var_at_prior_used: {}", prog.stack_def_use_live_var_at_prior_used.len());
    eprintln!("Execution: {:?}", run_time);
    eprintln!("\n=== Rule Times ===");
    let summary = prog.scc_times_summary();
    eprintln!("{}", summary);
    eprintln!("Total: {:?}", load_time + run_time);
}
