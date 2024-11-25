use std::{collections::BTreeMap, rc::Rc, usize};

use crate::{ascent_m_par, ascent_run_m_par};
use ascent::{ascent, internal::Convert};

type TupleID = usize;
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct Tag(&'static str, usize);

// data BindingList = Nil() | Cons(String, Exp, BindingList)
// data Exp = Num(Int) | Var(String) | Add(Exp, Exp) | Lam(String, Type, Exp) | App(Exp, Exp) | Let(String, Exp, Exp) | LetStar(BindingList, Exp)
// data Type = TInt() | TFun(Type, Type)
// data Ctx = Empty() | Bind(String, Type, Ctx)

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum BindingList {
   Nil,
   Cons(&'static str, Rc<Exp>, Rc<BindingList>),
}

type Ctx = BTreeMap<&'static str, Type>;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Exp {
   Num(i32),
   Var(&'static str),
   Add(Rc<Exp>, Rc<Exp>),
   Lam(&'static str, Type, Rc<Exp>),
   App(Rc<Exp>, Rc<Exp>),
   Let(&'static str, Rc<Exp>, Rc<Exp>),
   LetStar(BindingList, Rc<Exp>),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Type {
   TInt,
   TFun(Rc<Type>, Rc<Type>),
}

// #[derive(Debug, Clone, PartialEq, Eq, Hash)]
// enum Ctx {
//    Empty,
//    Bind(&'static str, Type, Rc<Ctx>),
// }

fn exp_tag(e: &Rc<Exp>, n: usize) -> Tag {
   match &**e {
      Exp::Num(_) => Tag("exp_num", n),
      Exp::Var(_) => Tag("exp_var", n),
      Exp::Add(_, _) => Tag("exp_add", n),
      Exp::Lam(_, _, _) => Tag("exp_lam", n),
      Exp::App(_, _) => Tag("exp_app", n),
      Exp::Let(_, _, _) => Tag("exp_let", n),
      Exp::LetStar(_, _) => Tag("exp_let_star", n),
   }
}

fn type_tag(t: &Rc<Type>, n: usize) -> Tag {
   match &**t {
      Type::TInt => Tag("type_prim", n),
      Type::TFun(_, _) => Tag("type_fun", n),
   }
}

fn binding_tag(b: &Rc<BindingList>, n: usize) -> Tag {
   match &**b {
      BindingList::Nil => Tag("binding_list_nil", n),
      BindingList::Cons(_, _, _) => Tag("binding_list_cons", n),
   }
}

fn type_rc_eq(t1: &Option<Type>, t2: &Rc<Type>) -> bool {
    match t1 {
        Some(t1) => t1 == &**t2,
        None => false,
    }
}

impl Convert<&Type> for Option<Type> {
    fn convert(t: &Type) -> Self {
        Some(t.clone())
    }
}

ascent! {
    struct IType;

    relation input_program(Exp);
    relation checked(Exp, Type);
    function type_of(Exp, Rc<Ctx>) -> Option<Type>;
    // function extend_ctx(Rc<Ctx>, &'static str, Type) -> Ctx;
    // function lookup(Rc<Ctx>, &'static str) -> Option<Type>;
    function type_eq(Type, Type) -> bool;

    %type_of(e, ctx) -> tt <--
        if let Exp::Num(_) = e,
        let tt = &Some(Type::TInt);
    %type_of(e, ctx) -> tt <--
        if let Exp::Var(x) = e,
        if let Some(tt) = ctx.get(x);
    %type_of(e, ctx) -> tt <--
        if let Exp::Add(e1_p, e2_p) = e, let e1 = &**e1_p,
        %type_of(e1, ctx) -> tt,
        if let None = tt;
    %type_of(e, ctx) -> tt <--
        if let Exp::Add(e1_p, e2_p) = e, let e2 = &**e2_p,
        %type_of(e, ctx) -> tt,
        if let None = tt;
    %type_of(e, ctx) -> res_type <--
        if let Exp::Add(e1_p, e2_p) = e, let e1 = &**e1_p, let e2 = &**e2_p,
        %type_of(e1, ctx) -> tt1,
        if let Some(Type::TInt) = tt1,
        let res_type = &None;
    %type_of(e, ctx) -> res_type <--
        if let Exp::Add(e1_p, e2_p) = e, let e1 = &**e1_p, let e2 = &**e2_p,
        %type_of(e1, ctx) -> tt1,
        %type_of(e2, ctx) -> tt2,
        if let Some(Type::TFun(t_arg, t_ret)) = tt1,
        ((if !type_rc_eq(tt2, t_arg),
         let res_type = &None) ||
         (if type_rc_eq(tt2, t_arg),
         let res_type = &Some((**t_ret).clone())));
    // %type_of(e_lam, ctx) -> tt <--
        // if let Exp::Lam(x, t_arg, body_p) = e_lam, let e = &**body_p,
        
        // %type_of(e, ctx_p) -> tt;
    
}

#[test]
fn test_itype() {
   // def program: Exp = Lam("m", TFun(TFun(TInt(), TInt()), TFun(TInt(), TInt())),
   //                        Lam("n", TFun(TFun(TInt(), TInt()), TFun(TInt(), TInt())),
   //                          Lam("f", TFun(TInt(), TInt()),
   //                            Lam("x", TInt(),
   //                              App(
   //                                App(Var("m"), Var("f")),
   //                                App(App(Var("n"), Var("f")), Var("x"))
   //                              )))))

   let test_program = Exp::Lam(
      "m",
      Type::TFun(
         Rc::new(Type::TFun(Rc::new(Type::TInt), Rc::new(Type::TInt))),
         Rc::new(Type::TFun(Rc::new(Type::TInt), Rc::new(Type::TInt))),
      ),
      Rc::new(Exp::Lam(
         "n",
         Type::TFun(
            Rc::new(Type::TFun(Rc::new(Type::TInt), Rc::new(Type::TInt))),
            Rc::new(Type::TFun(Rc::new(Type::TInt), Rc::new(Type::TInt))),
         ),
         Rc::new(Exp::Lam(
            "f",
            Type::TFun(Rc::new(Type::TInt), Rc::new(Type::TInt)),
            Rc::new(Exp::Lam(
               "x",
               Type::TInt,
               Rc::new(Exp::App(
                  Rc::new(Exp::App(Rc::new(Exp::Var("m")), Rc::new(Exp::Var("f")))),
                  Rc::new(Exp::App(
                     Rc::new(Exp::App(Rc::new(Exp::Var("n")), Rc::new(Exp::Var("f")))),
                     Rc::new(Exp::Var("x")),
                  )),
               )),
            )),
         )),
      )),
   );

   let mut flatten_db = IType::default();
//    flatten_db.input_program = vec![(Rc::new(test_program.clone()),)];

   flatten_db.run();
}
