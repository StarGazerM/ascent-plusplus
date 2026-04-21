// Doop Points-to Analysis — pure Ascent batch.
// Equivalent rule structure to SRDatalog doop_bg.nim. Uses negation, so
// not compatible with the DD batch-mode backend.
//
// Usage: doop_batch <input_dir> <abstract> <public> <static> <main> <main_desc>
//   <object> <cloneable> <serializable> <clinit> <clinit_desc> <class_init>
//   <reg_natives> <desired_assert> <object_array> <string_type>

use ascent::ascent_par;
use bench_loader::{load_1, load_2, load_3, load_4};
use std::env;
use std::time::Instant;

type V = i32;

ascent_par! {
    #![measure_rule_times]
    // EDB
    relation direct_superclass(V, V);
    relation direct_superinterface(V, V);
    relation main_class(V);
    relation formal_param(V, V, V);
    relation component_type(V, V);
    relation assign_return_value(V, V);
    relation actual_param(V, V, V);
    relation method_modifier(V, V);
    relation var_type(V, V);
    relation heap_alloc_type(V, V);
    relation method_descriptor(V, V);
    relation class_type(V);
    relation array_type(V);
    relation interface_type(V);
    relation application_class(V);
    relation this_var(V, V);
    relation field_declaring_type(V, V);
    relation method_simple_name(V, V);
    relation method_declaring_type(V, V);
    relation instruction_method(V, V);
    relation is_virtual_method_inv_insn(V);
    relation is_static_method_inv_insn(V);
    relation method_inv_method(V, V);
    relation vmi_base(V, V);
    relation smi_base(V, V);
    relation vmi_simple_name(V, V);
    relation vmi_descriptor(V, V);
    relation load_instance_field(V, V, V, V);
    relation store_instance_field(V, V, V, V);
    relation load_static_field(V, V, V);
    relation store_static_field(V, V, V);
    relation load_array_index(V, V, V);
    relation store_array_index(V, V, V);
    relation assign_cast(V, V, V, V);
    relation assign_local(V, V, V);
    relation assign_heap_alloc(V, V, V);
    relation return_var(V, V);
    relation static_method_inv(V, V, V);

    // Constants as single-value relations
    relation c_abstract(V);
    relation c_public(V);
    relation c_static(V);
    relation c_main(V);
    relation c_main_desc(V);
    relation c_object(V);
    relation c_cloneable(V);
    relation c_serializable(V);
    relation c_clinit(V);
    relation c_clinit_desc(V);
    relation c_class_init(V);
    relation c_reg_natives(V);
    relation c_desired_assert(V);
    relation c_object_array(V);
    relation c_string_type(V);

    // IDB
    relation is_type(V);
    relation is_ref_type(V);
    relation is_array_type_idb(V);
    relation is_class_type_idb(V);
    relation is_interface_type_idb(V);
    relation method_lookup(V, V, V, V);
    relation method_implemented(V, V, V, V);
    relation direct_subclass(V, V);
    relation subclass(V, V);
    relation superinterface(V, V);
    relation subtype_of(V, V);
    relation supertype_of(V, V);
    relation main_method_decl(V);
    relation class_initializer(V, V);
    relation initialized_class(V);
    relation cast_to(V, V, V, V);
    relation heap_helper(V, V, V, V, V);
    relation heap_helper_no_this(V, V, V, V);
    relation precomputed_vmi(V, V, V, V);
    relation heap_alloc_super_type(V, V);
    relation is_object_array_heap(V);
    relation is_string_heap(V);
    relation is_castable_to_string(V);
    relation array_type_compat(V, V);
    relation reachable_instruction(V);
    relation reachable_load_instance_field(V, V, V);
    relation reachable_sorted_index(V, V);
    relation assign_rel(V, V);
    relation var_points_to(V, V);
    relation instance_field_points_to(V, V, V);
    relation static_field_points_to(V, V);
    relation call_graph_edge(V, V);
    relation array_index_points_to(V, V);
    relation reachable(V);

    // Type system
    is_type(c) <-- class_type(c);
    is_ref_type(c) <-- class_type(c);
    is_class_type_idb(c) <-- class_type(c);
    is_type(a) <-- array_type(a);
    is_ref_type(a) <-- array_type(a);
    is_array_type_idb(a) <-- array_type(a);
    is_type(i) <-- interface_type(i);
    is_ref_type(i) <-- interface_type(i);
    is_interface_type_idb(i) <-- interface_type(i);
    is_type(t) <-- application_class(t);
    is_ref_type(t) <-- application_class(t);

    superinterface(k, c) <-- direct_superinterface(c, k);
    superinterface(k, c) <-- superinterface(k, b), direct_superinterface(c, b);
    superinterface(k, c) <-- superinterface(k, b), direct_superclass(c, b);

    method_implemented(sn, desc, mtype, method) <--
        method_simple_name(method, sn), method_descriptor(method, desc),
        method_declaring_type(method, mtype),
        c_abstract(?abs), !method_modifier(abs, method);

    main_method_decl(method) <--
        main_class(mtype), method_declaring_type(method, mtype),
        c_main(?m), method_simple_name(method, m),
        c_main_desc(?md), method_descriptor(method, md),
        c_public(?p), method_modifier(p, method),
        c_static(?s), method_modifier(s, method),
        c_class_init(?ci), c_reg_natives(?rn), c_desired_assert(?da),
        if *method != *ci && *method != *rn && *method != *da;

    direct_subclass(a, c) <-- direct_superclass(a, c);
    subclass(c, a) <-- direct_subclass(a, c);
    subclass(c, a) <-- subclass(b, a), direct_subclass(b, c);

    method_lookup(sn, desc, mtype, method) <-- method_implemented(sn, desc, mtype, method);
    method_lookup(sn, desc, mtype, method) <--
        direct_superclass(mtype, sup), method_lookup(sn, desc, sup, method),
        !method_implemented(sn, desc, mtype, _);
    method_lookup(sn, desc, mtype, method) <--
        direct_superinterface(mtype, sup), method_lookup(sn, desc, sup, method),
        !method_implemented(sn, desc, mtype, _);

    subtype_of(s, s) <-- is_class_type_idb(s);
    subtype_of(t, t) <-- is_type(t);
    subtype_of(s, s) <-- is_interface_type_idb(s);
    subtype_of(s, t) <-- subclass(t, s);
    subtype_of(s, t) <-- is_class_type_idb(s), superinterface(t, s);
    subtype_of(s, t) <-- is_interface_type_idb(s), superinterface(t, s);
    subtype_of(s, *obj) <-- is_interface_type_idb(s), c_object(obj);
    subtype_of(s, *obj) <-- is_array_type_idb(s), c_object(obj);
    subtype_of(s, t) <-- subtype_of(sc, tc), component_type(s, sc), component_type(t, tc),
        is_ref_type(sc), is_ref_type(tc);
    subtype_of(s, *clon) <-- is_array_type_idb(s), c_cloneable(clon);
    subtype_of(s, *ser) <-- is_array_type_idb(s), c_serializable(ser);

    supertype_of(s, t) <-- subtype_of(t, s);

    class_initializer(ctype, method) <--
        c_clinit(?cl), c_clinit_desc(?cld),
        method_implemented(cl, cld, ctype, method);

    // Precomputes — CastTo excluding String heaps
    cast_to(frm, to, inmeth, heap) <--
        assign_cast(casttype, frm, to, inmeth), supertype_of(casttype, heaptype),
        heap_alloc_type(heap, heaptype), c_string_type(?st), if *heaptype != *st;
    cast_to(frm, to, inmeth, heap) <--
        assign_cast(casttype, frm, to, inmeth), is_castable_to_string(casttype), is_string_heap(heap);

    heap_helper(sn, desc, to_meth, this_p, heap) <--
        method_lookup(sn, desc, heaptype, to_meth), heap_alloc_type(heap, heaptype), this_var(to_meth, this_p);
    heap_helper_no_this(sn, desc, to_meth, heap) <--
        method_lookup(sn, desc, heaptype, to_meth), heap_alloc_type(heap, heaptype);
    precomputed_vmi(inv, base, sn, desc) <--
        vmi_base(inv, base), vmi_simple_name(inv, sn), vmi_descriptor(inv, desc);

    heap_alloc_super_type(heap, baseheap) <--
        heap_alloc_type(heap, ht), heap_alloc_type(baseheap, bht),
        component_type(bht, comp), supertype_of(comp, ht),
        c_object_array(?oa), if *bht != *oa;
    is_object_array_heap(bh) <-- heap_alloc_type(bh, oa), c_object_array(?oa2), if *oa == *oa2;
    is_string_heap(h) <-- heap_alloc_type(h, st), c_string_type(?st2), if *st == *st2;
    is_castable_to_string(ct) <-- supertype_of(ct, st), c_string_type(?st2), if *st == *st2;
    array_type_compat(bh, vtype) <--
        heap_alloc_type(bh, bht), component_type(bht, bct), supertype_of(vtype, bct);

    reachable_instruction(inv) <-- reachable(im), instruction_method(inv, im);
    reachable_load_instance_field(base, sig, to) <-- reachable(im), load_instance_field(base, sig, to, im);
    reachable_sorted_index(frm, base) <-- reachable(im), store_array_index(frm, base, im);

    // Fixpoint
    initialized_class(sup) <-- initialized_class(cls), direct_superclass(cls, sup);
    initialized_class(sup) <-- initialized_class(cls), direct_superinterface(cls, sup);
    initialized_class(cls) <-- main_method_decl(m), method_declaring_type(m, cls);
    initialized_class(cls) <-- reachable(im), assign_heap_alloc(heap, _, im), heap_alloc_type(heap, cls);
    initialized_class(cls) <-- reachable(im), instruction_method(inv, im),
        is_static_method_inv_insn(inv), method_inv_method(inv, sig), method_declaring_type(sig, cls);
    initialized_class(cls) <-- reachable(im), store_static_field(_, sig, im), field_declaring_type(sig, cls);
    initialized_class(cls) <-- reachable(im), load_static_field(sig, _, im), field_declaring_type(sig, cls);
    reachable(clinit) <-- initialized_class(cls), class_initializer(cls, clinit);
    reachable(method) <-- main_method_decl(method);

    reachable(to) <-- reachable(im), static_method_inv(inv, to, im);
    call_graph_edge(inv, to) <-- reachable(im), static_method_inv(inv, to, im);

    assign_rel(actual, formal) <-- call_graph_edge(inv, method), formal_param(idx, method, formal), actual_param(idx, inv, actual);
    assign_rel(ret, local) <-- call_graph_edge(inv, method), return_var(ret, method), assign_return_value(inv, local);

    var_points_to(heap, var) <-- assign_heap_alloc(heap, var, im), reachable(im);
    var_points_to(heap, to) <-- assign_rel(from, to), var_points_to(heap, from);
    var_points_to(heap, to) <-- reachable(im), assign_local(from, to, im), var_points_to(heap, from);
    var_points_to(heap, to) <-- reachable(im), cast_to(frm, to, im, heap), var_points_to(heap, frm);
    var_points_to(heap, to) <-- reachable_load_instance_field(base, sig, to),
        var_points_to(bh, base), instance_field_points_to(heap, sig, bh);
    var_points_to(heap, to) <-- reachable(im), load_static_field(fld, to, im), static_field_points_to(heap, fld);
    var_points_to(heap, to) <-- reachable(im), load_array_index(base, to, im),
        var_points_to(bh, base), array_index_points_to(bh, heap),
        var_type(to, vtype), array_type_compat(bh, vtype);

    var_points_to(heap, this_p) <-- reachable_instruction(inv), precomputed_vmi(inv, base, sn, desc),
        var_points_to(heap, base), heap_helper(sn, desc, to_meth, this_p, heap);
    call_graph_edge(inv, to_meth) <-- reachable_instruction(inv), precomputed_vmi(inv, base, sn, desc),
        var_points_to(heap, base), heap_helper(sn, desc, to_meth, _tp, heap);
    reachable(to_meth) <-- reachable_instruction(inv), precomputed_vmi(inv, base, sn, desc),
        var_points_to(heap, base), heap_helper(sn, desc, to_meth, _tp, heap);
    reachable(to_meth) <-- reachable_instruction(inv), precomputed_vmi(inv, base, sn, desc),
        var_points_to(heap, base), heap_helper_no_this(sn, desc, to_meth, heap);
    call_graph_edge(inv, to_meth) <-- reachable_instruction(inv), precomputed_vmi(inv, base, sn, desc),
        var_points_to(heap, base), heap_helper_no_this(sn, desc, to_meth, heap);

    var_points_to(heap, tv) <-- reachable_instruction(inv), smi_base(inv, base),
        var_points_to(heap, base), method_inv_method(inv, to), this_var(to, tv);
    call_graph_edge(inv, to) <-- reachable_instruction(inv), smi_base(inv, base),
        var_points_to(heap, base), method_inv_method(inv, to), this_var(to, _tv);
    reachable(to) <-- reachable_instruction(inv), smi_base(inv, base),
        var_points_to(heap, base), method_inv_method(inv, to), this_var(to, _tv);

    instance_field_points_to(heap, fld, bh) <-- reachable(im), store_instance_field(from, base, fld, im),
        var_points_to(heap, from), var_points_to(bh, base);
    static_field_points_to(heap, fld) <-- reachable(im), store_static_field(from, fld, im), var_points_to(heap, from);

    array_index_points_to(bh, heap) <-- reachable_sorted_index(frm, base),
        var_points_to(bh, base), is_object_array_heap(bh), var_points_to(heap, frm);
    array_index_points_to(bh, heap) <-- reachable_sorted_index(frm, base),
        var_points_to(bh, base), var_points_to(heap, frm), heap_alloc_super_type(heap, bh);
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 17 {
        eprintln!("Usage: {} <dir> <abstract> <public> <static> <main> <main_desc> <object> <cloneable> <serializable> <clinit> <clinit_desc> <class_init> <reg_natives> <desired_assert> <object_array> <string_type>", args[0]);
        std::process::exit(1);
    }
    let dir = &args[1];
    let p = |i: usize| -> V { args[i].parse().unwrap() };

    eprintln!("Loading data from {}...", dir);
    let t0 = Instant::now();
    let mut prog = AscentProgram::default();

    macro_rules! load_into {
        ($rel:expr, $data:expr) => { for t in $data { $rel.push(t); } }
    }

    load_into!(prog.direct_superclass, load_2(dir, "DirectSuperclass.csv"));
    load_into!(prog.direct_superinterface, load_2(dir, "DirectSuperinterface.csv"));
    load_into!(prog.main_class, load_1(dir, "MainClass.csv"));
    load_into!(prog.formal_param, load_3(dir, "FormalParam.csv"));
    load_into!(prog.component_type, load_2(dir, "ComponentType.csv"));
    load_into!(prog.assign_return_value, load_2(dir, "AssignReturnValue.csv"));
    load_into!(prog.actual_param, load_3(dir, "ActualParam.csv"));
    load_into!(prog.method_modifier, load_2(dir, "Method_Modifier.csv"));
    load_into!(prog.var_type, load_2(dir, "Var_Type.csv"));
    load_into!(prog.heap_alloc_type, load_2(dir, "HeapAllocation_Type.csv"));
    load_into!(prog.method_descriptor, load_2(dir, "Method_Descriptor.csv"));
    load_into!(prog.class_type, load_1(dir, "ClassType.csv"));
    load_into!(prog.array_type, load_1(dir, "ArrayType.csv"));
    load_into!(prog.interface_type, load_1(dir, "InterfaceType.csv"));
    load_into!(prog.application_class, load_1(dir, "ApplicationClass.csv"));
    load_into!(prog.this_var, load_2(dir, "ThisVar.csv"));
    load_into!(prog.field_declaring_type, load_2(dir, "Field_DeclaringType.csv"));
    load_into!(prog.method_simple_name, load_2(dir, "Method_SimpleName.csv"));
    load_into!(prog.method_declaring_type, load_2(dir, "Method_DeclaringType.csv"));
    load_into!(prog.instruction_method, load_2(dir, "Instruction_Method.csv"));
    load_into!(prog.is_virtual_method_inv_insn, load_1(dir, "isVirtualMethodInvocation_Insn.csv"));
    load_into!(prog.is_static_method_inv_insn, load_1(dir, "isStaticMethodInvocation_Insn.csv"));
    load_into!(prog.method_inv_method, load_2(dir, "MethodInvocation_Method.csv"));
    load_into!(prog.vmi_base, load_2(dir, "VirtualMethodInvocation_Base.csv"));
    load_into!(prog.smi_base, load_2(dir, "SpecialMethodInvocation_Base.csv"));
    load_into!(prog.vmi_simple_name, load_2(dir, "VirtualMethodInvocation_SimpleName.csv"));
    load_into!(prog.vmi_descriptor, load_2(dir, "VirtualMethodInvocation_Descriptor.csv"));
    load_into!(prog.load_instance_field, load_4(dir, "LoadInstanceField.csv"));
    load_into!(prog.store_instance_field, load_4(dir, "StoreInstanceField.csv"));
    load_into!(prog.load_static_field, load_3(dir, "LoadStaticField.csv"));
    load_into!(prog.store_static_field, load_3(dir, "StoreStaticField.csv"));
    load_into!(prog.load_array_index, load_3(dir, "LoadArrayIndex.csv"));
    load_into!(prog.store_array_index, load_3(dir, "StoreArrayIndex.csv"));
    load_into!(prog.assign_cast, load_4(dir, "AssignCast.csv"));
    load_into!(prog.assign_local, load_3(dir, "AssignLocal.csv"));
    load_into!(prog.assign_heap_alloc, load_3(dir, "AssignHeapAllocation.csv"));
    load_into!(prog.return_var, load_2(dir, "ReturnVar.csv"));
    load_into!(prog.static_method_inv, load_3(dir, "StaticMethodInvocation.csv"));

    prog.c_abstract.push((p(2),));
    prog.c_public.push((p(3),));
    prog.c_static.push((p(4),));
    prog.c_main.push((p(5),));
    prog.c_main_desc.push((p(6),));
    prog.c_object.push((p(7),));
    prog.c_cloneable.push((p(8),));
    prog.c_serializable.push((p(9),));
    prog.c_clinit.push((p(10),));
    prog.c_clinit_desc.push((p(11),));
    prog.c_class_init.push((p(12),));
    prog.c_reg_natives.push((p(13),));
    prog.c_desired_assert.push((p(14),));
    prog.c_object_array.push((p(15),));
    prog.c_string_type.push((p(16),));

    let load_time = t0.elapsed();
    eprintln!("Loaded in {:?}", load_time);

    let t1 = Instant::now();
    prog.run();
    let run_time = t1.elapsed();

    eprintln!("var_points_to: {}", prog.var_points_to.len());
    eprintln!("instance_field_points_to: {}", prog.instance_field_points_to.len());
    eprintln!("call_graph_edge: {}", prog.call_graph_edge.len());
    eprintln!("array_index_points_to: {}", prog.array_index_points_to.len());
    eprintln!("reachable: {}", prog.reachable.len());
    eprintln!("Execution: {:?}", run_time);
    eprintln!("\n=== Rule Times ===");
    let summary = prog.scc_times_summary();
    eprintln!("{}", summary);
    eprintln!("Total: {:?}", load_time + run_time);
}
