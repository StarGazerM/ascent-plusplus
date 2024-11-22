// suppress all warnings
#![allow(warnings)]

use ascent::aggregators::{count, max, min, sum};
use ascent::ascent;

use std::cmp;
use std::cmp::Ordering;
use std::fs::File;
use std::hash::{Hash, Hasher};
use std::ops;
use std::str::FromStr;
// // Used for serialization to and from JSON
// use serde::ser::SerializeSeq;
// use serde::{Serialize, Serializer};

macro_rules! format_error_msg {
    ($($args:expr),*) => {
        format!("Failed to parse field {} into expected type", $($args),*)
    };
}

// #[derive(Debug, Clone, Copy, Serialize)]
#[derive(Debug, Clone, Copy)]
pub struct Float(f32);
impl Hash for Float {
   fn hash<H: Hasher>(&self, state: &mut H) {
      let mut buf = [0; 4];
      let transmuted: u32 = unsafe { std::mem::transmute(self.0) };
      buf.copy_from_slice(&transmuted.to_le_bytes());
      buf.hash(state)
   }
}
impl PartialEq for Float {
   fn eq(&self, other: &Float) -> bool { self.0 == other.0 }
}
impl Eq for Float {}
impl FromStr for Float {
   type Err = std::num::ParseFloatError;
   fn from_str(s: &str) -> Result<Self, Self::Err> {
      let parsed = s.parse::<f32>()?;
      Ok(Float(parsed))
   }
}
impl ops::Add for Float {
   type Output = Float;
   fn add(self, other: Float) -> Float { Float(self.0 + other.0) }
}
impl ops::Sub for Float {
   type Output = Float;
   fn sub(self, other: Float) -> Float { Float(self.0 - other.0) }
}
impl ops::Mul for Float {
   type Output = Float;
   fn mul(self, other: Float) -> Float { Float(self.0 * other.0) }
}
impl ops::Div for Float {
   type Output = Float;
   fn div(self, other: Float) -> Float { Float(self.0 / other.0) }
}
impl ops::Rem for Float {
   type Output = Float;
   fn rem(self, other: Float) -> Float { Float(self.0 % other.0) }
}
impl Ord for Float {
   fn cmp(&self, other: &Self) -> std::cmp::Ordering {
      self.0.partial_cmp(&other.0).unwrap_or(std::cmp::Ordering::Equal)
   }
}
impl PartialOrd for Float {
   fn partial_cmp(&self, other: &Float) -> Option<std::cmp::Ordering> { Some(self.cmp(&other)) }
}
impl std::ops::AddAssign for Float {
   fn add_assign(&mut self, other: Self) { self.0 += other.0; }
}
impl std::iter::Sum for Float {
   fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
      let mut sum = Float(0.0);
      for item in iter {
         sum += item;
      }
      sum
   }
}

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum BindingList {
   Nil(),
   Cons(String, Box<Exp>, Box<BindingList>),
}

fn destruct_Nil(obj: BindingList) -> Option<()> {
   return match obj {
      BindingList::Nil() => Some(()),
      BindingList::Cons(_, _, _) => None,
   };
}

fn destruct_Cons(obj: BindingList) -> Option<(String, Exp, BindingList)> {
   return match obj {
      BindingList::Nil() => None,
      BindingList::Cons(param_0, param_1, param_2) => Some((param_0.to_string(), *param_1, *param_2)),
   };
}

// Do not serialize custom data types. Otherwise it will get to slow
// impl Serialize for BindingList {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//       S: Serializer,
//    {
//       serializer.serialize_str(&format!("{:?}", &self))
//    }
// }

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Exp {
   Num(i32),
   Var(String),
   Add(Box<Exp>, Box<Exp>),
   Lam(String, Type, Box<Exp>),
   App(Box<Exp>, Box<Exp>),
   Let(String, Box<Exp>, Box<Exp>),
   LetStar(Box<BindingList>, Box<Exp>),
}

fn destruct_Num(obj: Exp) -> Option<i32> {
   return match obj {
      Exp::Num(param_0) => Some(param_0),
      Exp::Var(_) => None,
      Exp::Add(_, _) => None,
      Exp::Lam(_, _, _) => None,
      Exp::App(_, _) => None,
      Exp::Let(_, _, _) => None,
      Exp::LetStar(_, _) => None,
   };
}

fn destruct_Var(obj: Exp) -> Option<String> {
   return match obj {
      Exp::Num(_) => None,
      Exp::Var(param_0) => Some(param_0.to_string()),
      Exp::Add(_, _) => None,
      Exp::Lam(_, _, _) => None,
      Exp::App(_, _) => None,
      Exp::Let(_, _, _) => None,
      Exp::LetStar(_, _) => None,
   };
}

fn destruct_Add(obj: Exp) -> Option<(Exp, Exp)> {
   return match obj {
      Exp::Num(_) => None,
      Exp::Var(_) => None,
      Exp::Add(param_0, param_1) => Some((*param_0, *param_1)),
      Exp::Lam(_, _, _) => None,
      Exp::App(_, _) => None,
      Exp::Let(_, _, _) => None,
      Exp::LetStar(_, _) => None,
   };
}

fn destruct_Lam(obj: Exp) -> Option<(String, Type, Exp)> {
   return match obj {
      Exp::Num(_) => None,
      Exp::Var(_) => None,
      Exp::Add(_, _) => None,
      Exp::Lam(param_0, param_1, param_2) => Some((param_0.to_string(), param_1, *param_2)),
      Exp::App(_, _) => None,
      Exp::Let(_, _, _) => None,
      Exp::LetStar(_, _) => None,
   };
}

fn destruct_App(obj: Exp) -> Option<(Exp, Exp)> {
   return match obj {
      Exp::Num(_) => None,
      Exp::Var(_) => None,
      Exp::Add(_, _) => None,
      Exp::Lam(_, _, _) => None,
      Exp::App(param_0, param_1) => Some((*param_0, *param_1)),
      Exp::Let(_, _, _) => None,
      Exp::LetStar(_, _) => None,
   };
}

fn destruct_Let(obj: Exp) -> Option<(String, Exp, Exp)> {
   return match obj {
      Exp::Num(_) => None,
      Exp::Var(_) => None,
      Exp::Add(_, _) => None,
      Exp::Lam(_, _, _) => None,
      Exp::App(_, _) => None,
      Exp::Let(param_0, param_1, param_2) => Some((param_0.to_string(), *param_1, *param_2)),
      Exp::LetStar(_, _) => None,
   };
}

fn destruct_LetStar(obj: Exp) -> Option<(BindingList, Exp)> {
   return match obj {
      Exp::Num(_) => None,
      Exp::Var(_) => None,
      Exp::Add(_, _) => None,
      Exp::Lam(_, _, _) => None,
      Exp::App(_, _) => None,
      Exp::Let(_, _, _) => None,
      Exp::LetStar(param_0, param_1) => Some((*param_0, *param_1)),
   };
}

// Do not serialize custom data types. Otherwise it will get to slow
// impl Serialize for Exp {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//       S: Serializer,
//    {
//       serializer.serialize_str(&format!("{:?}", &self))
//    }
// }

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Type {
   TInt(),
   TFun(Box<Type>, Box<Type>),
}

fn destruct_TInt(obj: Type) -> Option<()> {
   return match obj {
      Type::TInt() => Some(()),
      Type::TFun(_, _) => None,
   };
}

fn destruct_TFun(obj: Type) -> Option<(Type, Type)> {
   return match obj {
      Type::TInt() => None,
      Type::TFun(param_0, param_1) => Some((*param_0, *param_1)),
   };
}

// Do not serialize custom data types. Otherwise it will get to slow
// impl Serialize for Type {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//       S: Serializer,
//    {
//       serializer.serialize_str(&format!("{:?}", &self))
//    }
// }

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Ctx {
   Empty(),
   Bind(String, Type, Box<Ctx>),
}

fn destruct_Empty(obj: Ctx) -> Option<()> {
   return match obj {
      Ctx::Empty() => Some(()),
      Ctx::Bind(_, _, _) => None,
   };
}

fn destruct_Bind(obj: Ctx) -> Option<(String, Type, Ctx)> {
   return match obj {
      Ctx::Empty() => None,
      Ctx::Bind(param_0, param_1, param_2) => Some((param_0.to_string(), param_1, *param_2)),
   };
}

// Do not serialize custom data types. Otherwise it will get to slow
// impl Serialize for Ctx {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//       S: Serializer,
//    {
//       serializer.serialize_str(&format!("{:?}", &self))
//    }
// }

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Option_Type {
   Some_Type(Type),
   None_Type(),
}

fn destruct_Some_Type(obj: Option_Type) -> Option<Type> {
   return match obj {
      Option_Type::Some_Type(param_0) => Some(param_0),
      Option_Type::None_Type() => None,
   };
}

fn destruct_None_Type(obj: Option_Type) -> Option<()> {
   return match obj {
      Option_Type::Some_Type(_) => None,
      Option_Type::None_Type() => Some(()),
   };
}

// Do not serialize custom data types. Otherwise it will get to slow
// impl Serialize for Option_Type {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//       S: Serializer,
//    {
//       serializer.serialize_str(&format!("{:?}", &self))
//    }
// }

#[derive(Debug, Eq, PartialEq, Clone, Hash)]
pub enum Option_Ctx {
   Some_Ctx(Ctx),
   None_Ctx(),
}

fn destruct_Some_Ctx(obj: Option_Ctx) -> Option<Ctx> {
   return match obj {
      Option_Ctx::Some_Ctx(param_0) => Some(param_0),
      Option_Ctx::None_Ctx() => None,
   };
}

fn destruct_None_Ctx(obj: Option_Ctx) -> Option<()> {
   return match obj {
      Option_Ctx::Some_Ctx(_) => None,
      Option_Ctx::None_Ctx() => Some(()),
   };
}

// Do not serialize custom data types. Otherwise it will get to slow
// impl Serialize for Option_Ctx {
//    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//    where
//       S: Serializer,
//    {
//       serializer.serialize_str(&format!("{:?}", &self))
//    }
// }

ascent! {
  relation lookup_input(Ctx, String);
  lookup_input(ctx_0.clone(), n_0.clone()) <--
    typeOf_input(ctx_0, exp),
    let _tmp_0 = destruct_Var(exp.clone()),
    if !_tmp_0.is_none(), let n_0=_tmp_0.unwrap();
  lookup_input(ctx_0.clone(), n_0.clone()) <--
    lookup_input(ctx, n_0),
    let _tmp_0 = destruct_Bind(ctx.clone()),
    if !_tmp_0.is_none(), let (n1, ty, ctx_0)=_tmp_0.unwrap(), if *n_0.clone() != n1.clone();
  relation typeOf(Ctx, Exp, Option_Type);
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Num(exp.clone()),
    if !_tmp_0.is_none(),
    let v=_tmp_0.unwrap(),
    let typeOf_result_0 = Option_Type::Some_Type(Type::TInt());
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Var(exp.clone()),
    if !_tmp_0.is_none(),
    let n=_tmp_0.unwrap(),
    lookup(ctx, n, lookup_call_0),
    let typeOf_result_0 = lookup_call_0.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Add(exp.clone()),
    if !_tmp_0.is_none(),
    let (l, r)=_tmp_0.unwrap(),
    typeOf(ctx, l, typeOf_call_1),
    let _tmp_1 = destruct_None_Type(typeOf_call_1.clone()),
    if !_tmp_1.is_none(),
    let ()=_tmp_1.unwrap(),
    let match_result_1 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_1.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Add(exp.clone()),
    if !_tmp_0.is_none(),
    let (l, r)=_tmp_0.unwrap(),
    typeOf(ctx, l, typeOf_call_1),
    let _tmp_1 = destruct_Some_Type(typeOf_call_1.clone()),
    if !_tmp_1.is_none(),
    let lty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TInt(lty.clone()),
    if !_tmp_2.is_none(),
    let ()=_tmp_2.unwrap(),
    typeOf(ctx, r, typeOf_call_2),
    let _tmp_3 = destruct_None_Type(typeOf_call_2.clone()),
    if !_tmp_3.is_none(),
    let ()=_tmp_3.unwrap(), 
    let match_result_3 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_3.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Add(exp.clone()),
    if !_tmp_0.is_none(), let (l, r)=_tmp_0.unwrap(),
    typeOf(ctx, l, typeOf_call_1),
    let _tmp_1 = destruct_Some_Type(typeOf_call_1.clone()),
    if !_tmp_1.is_none(), let lty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TInt(lty.clone()),
    if !_tmp_2.is_none(), let ()=_tmp_2.unwrap(),
    typeOf(ctx, r, typeOf_call_2),
    let _tmp_3 = destruct_Some_Type(typeOf_call_2.clone()),
    if !_tmp_3.is_none(), let rty=_tmp_3.unwrap(),
    let _tmp_4 = destruct_TInt(rty.clone()),
    if !_tmp_4.is_none(), let ()=_tmp_4.unwrap(),
    let match_result_4 = Option_Type::Some_Type(Type::TInt()),
    let typeOf_result_0 = match_result_4.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Add(exp.clone()), if !_tmp_0.is_none(),
    let (l, r)=_tmp_0.unwrap(),
    typeOf(ctx, l, typeOf_call_1),
    let _tmp_1 = destruct_Some_Type(typeOf_call_1.clone()),
    if !_tmp_1.is_none(), let lty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TInt(lty.clone()),
    if !_tmp_2.is_none(), let ()=_tmp_2.unwrap(),
    typeOf(ctx, r, typeOf_call_2),
    let _tmp_3 = destruct_Some_Type(typeOf_call_2.clone()),
    if !_tmp_3.is_none(), let rty=_tmp_3.unwrap(),
    let _tmp_4 = destruct_TFun(rty.clone()),
    if !_tmp_4.is_none(), let (ty1, ty2)=_tmp_4.unwrap(),
    let match_result_4 = Option_Type::None_Type(), let typeOf_result_0 = match_result_4.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp), 
    let _tmp_0 = destruct_Add(exp.clone()),
    if !_tmp_0.is_none(), let (l, r)=_tmp_0.unwrap(),
    typeOf(ctx, l, typeOf_call_1),
    let _tmp_1 = destruct_Some_Type(typeOf_call_1.clone()),
    if !_tmp_1.is_none(), let lty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TFun(lty.clone()),
    if !_tmp_2.is_none(), let (ty1, ty2)=_tmp_2.unwrap(),
    let match_result_2 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_2.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Lam(exp.clone()),
    if !_tmp_0.is_none(), let (n, ty, b)=_tmp_0.unwrap(),
    typeOf(Ctx::Bind((n.clone()).to_string(),ty.clone(),Box::new(ctx.clone())), b, typeOf_call_3),
    let _tmp_1 = destruct_None_Type(typeOf_call_3.clone()),
    if !_tmp_1.is_none(),let ()=_tmp_1.unwrap(),
    let match_result_5 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_5.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Lam(exp.clone()),
    if !_tmp_0.is_none(), let (n, ty, b)=_tmp_0.unwrap(),
    typeOf(Ctx::Bind((n.clone()).to_string(),ty.clone(),Box::new(ctx.clone())), b, typeOf_call_3),
    let _tmp_1 = destruct_Some_Type(typeOf_call_3.clone()),
    if !_tmp_1.is_none(), let ty2=_tmp_1.unwrap(),
    let match_result_5 = Option_Type::Some_Type(Type::TFun(Box::new(ty.clone()),Box::new(ty2.clone()))),
    let typeOf_result_0 = match_result_5.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_App(exp.clone()), if !_tmp_0.is_none(),
    let (fun, arg)=_tmp_0.unwrap(), typeOf(ctx, fun, typeOf_call_4),
    let _tmp_1 = destruct_None_Type(typeOf_call_4.clone()),
    if !_tmp_1.is_none(), let ()=_tmp_1.unwrap(),
    let match_result_6 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_6.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_App(exp.clone()),
    if !_tmp_0.is_none(), let (fun, arg)=_tmp_0.unwrap(),
    typeOf(ctx, fun, typeOf_call_4),
    let _tmp_1 = destruct_Some_Type(typeOf_call_4.clone()),
    if !_tmp_1.is_none(), let funty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TInt(funty.clone()),
    if !_tmp_2.is_none(),let ()=_tmp_2.unwrap(),
    let match_result_7 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_7.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_App(exp.clone()),
    if !_tmp_0.is_none(), let (fun, arg)=_tmp_0.unwrap(),
    typeOf(ctx, fun, typeOf_call_4),
    let _tmp_1 = destruct_Some_Type(typeOf_call_4.clone()),
    if !_tmp_1.is_none(), let funty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TFun(funty.clone()),
    if !_tmp_2.is_none(), let (ty1, ty2)=_tmp_2.unwrap(),
    typeOf(ctx, arg, typeOf_call_5),
    let _tmp_3 = destruct_None_Type(typeOf_call_5.clone()),
    if !_tmp_3.is_none(), let ()=_tmp_3.unwrap(),
    let match_result_8 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_8.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_App(exp.clone()),
    if !_tmp_0.is_none(), let (fun, arg)=_tmp_0.unwrap(),
    typeOf(ctx, fun, typeOf_call_4), 
    let _tmp_1 = destruct_Some_Type(typeOf_call_4.clone()),
    if !_tmp_1.is_none(), let funty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TFun(funty.clone()),
    if !_tmp_2.is_none(), let (ty1, ty2)=_tmp_2.unwrap(),
    typeOf(ctx, arg, typeOf_call_5),
    let _tmp_3 = destruct_Some_Type(typeOf_call_5.clone()),
    if !_tmp_3.is_none(), let argty=_tmp_3.unwrap(),
    eqType(argty, ty1, eqType_call_0),
    if *eqType_call_0 == 1,
    let if_result_0 = Option_Type::Some_Type(ty2.clone()),
    let typeOf_result_0 = if_result_0.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_App(exp.clone()),
    if !_tmp_0.is_none(), let (fun, arg)=_tmp_0.unwrap(),
    typeOf(ctx, fun, typeOf_call_4),
    let _tmp_1 = destruct_Some_Type(typeOf_call_4.clone()),
    if !_tmp_1.is_none(), let funty=_tmp_1.unwrap(),
    let _tmp_2 = destruct_TFun(funty.clone()),
    if !_tmp_2.is_none(), let (ty1, ty2)=_tmp_2.unwrap(),
    typeOf(ctx, arg, typeOf_call_5),
    let _tmp_3 = destruct_Some_Type(typeOf_call_5.clone()),
    if !_tmp_3.is_none(), let argty=_tmp_3.unwrap(),
    eqType(argty, ty1, eqType_call_0),
    if *eqType_call_0 == 0, let if_result_0 = Option_Type::None_Type(),
    let typeOf_result_0 = if_result_0.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Let(exp.clone()),
    if !_tmp_0.is_none(), let (n, bound, body)=_tmp_0.unwrap(),
    typeOf(ctx, bound, typeOf_call_6),
    let _tmp_1 = destruct_None_Type(typeOf_call_6.clone()),
    if !_tmp_1.is_none(), let ()=_tmp_1.unwrap(),
    let match_result_9 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_9.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_Let(exp.clone()),
    if !_tmp_0.is_none(), let (n, bound, body)=_tmp_0.unwrap(),
    typeOf(ctx, bound, typeOf_call_6),
    let _tmp_1 = destruct_Some_Type(typeOf_call_6.clone()),
    if !_tmp_1.is_none(), let boundty=_tmp_1.unwrap(),
    typeOf(Ctx::Bind((n.clone()).to_string(),
           boundty.clone(),Box::new(ctx.clone())),
           body, typeOf_call_7),
    let typeOf_result_0 = typeOf_call_7.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_LetStar(exp.clone()),
    if !_tmp_0.is_none(), let (bindings, body)=_tmp_0.unwrap(),
    extendCtx(ctx, bindings, extendCtx_call_0),
    let _tmp_1 = destruct_None_Ctx(extendCtx_call_0.clone()),
    if !_tmp_1.is_none(), let ()=_tmp_1.unwrap(),
    let match_result_10 = Option_Type::None_Type(),
    let typeOf_result_0 = match_result_10.clone();
  typeOf(ctx.clone(), exp.clone(), typeOf_result_0.clone()) <--
    typeOf_input(ctx, exp),
    let _tmp_0 = destruct_LetStar(exp.clone()), if !_tmp_0.is_none(),
    let (bindings, body)=_tmp_0.unwrap(),
    extendCtx(ctx, bindings, extendCtx_call_0),
    let _tmp_1 = destruct_Some_Ctx(extendCtx_call_0.clone()),
    if !_tmp_1.is_none(), let extCtx=_tmp_1.unwrap(),
    typeOf(extCtx, body, typeOf_call_8),
    let typeOf_result_0 = typeOf_call_8.clone();
  
  
  relation eqType(Type, Type, i32);
  eqType(ty1.clone(), ty2.clone(), eqType_result_0) <--
    eqType_input(ty1, ty2),
    let _tmp_0 = destruct_TInt(ty1.clone()), if !_tmp_0.is_none(),
    let ()=_tmp_0.unwrap(), let _tmp_1 = destruct_TInt(ty2.clone()),
    if !_tmp_1.is_none(), let ()=_tmp_1.unwrap(),
    let eqType_result_0 = 1;
  eqType(ty1.clone(), ty2.clone(), eqType_result_0) <--
    eqType_input(ty1, ty2),
    let _tmp_0 = destruct_TInt(ty1.clone()),
    if !_tmp_0.is_none(), let ()=_tmp_0.unwrap(),
    let _tmp_1 = destruct_TFun(ty2.clone()),
    if !_tmp_1.is_none(), let (ofty1, ofty2)=_tmp_1.unwrap(),
    let eqType_result_0 = 0;
  eqType(ty1.clone(), ty2.clone(), eqType_result_0) <--
    eqType_input(ty1, ty2),
    let _tmp_0 = destruct_TFun(ty1.clone()),
    if !_tmp_0.is_none(), let (fty1, fty2)=_tmp_0.unwrap(),
    let _tmp_1 = destruct_TInt(ty2.clone()),
    if !_tmp_1.is_none(), let ()=_tmp_1.unwrap(),
    let eqType_result_0 = 0;
eqType(ty1.clone(), ty2.clone(), eqType_result_0) <-- eqType_input(ty1, ty2), let _tmp_0 = destruct_TFun(ty1.clone()), if !_tmp_0.is_none(), let (fty1, fty2)=_tmp_0.unwrap(), let _tmp_1 = destruct_TFun(ty2.clone()), if !_tmp_1.is_none(), let (ofty1, ofty2)=_tmp_1.unwrap(), eqType(fty1, ofty1, eqType_call_1), eqType(fty2, ofty2, eqType_call_2), let match_result_16 = cmp::min(*eqType_call_1, *eqType_call_2), let eqType_result_0 = match_result_16;
relation lookup(Ctx, String, Option_Type);
lookup(ctx.clone(), n.clone(), lookup_result_0.clone()) <-- lookup_input(ctx, n), let _tmp_0 = destruct_Empty(ctx.clone()), if !_tmp_0.is_none(), let ()=_tmp_0.unwrap(), let lookup_result_0 = Option_Type::None_Type();
lookup(ctx.clone(), n.clone(), lookup_result_0.clone()) <-- lookup_input(ctx, n), let _tmp_0 = destruct_Bind(ctx.clone()), if !_tmp_0.is_none(), let (n1, ty, rest)=_tmp_0.unwrap(), if n1 == *n, let if_result_1 = Option_Type::Some_Type(ty.clone()), let lookup_result_0 = if_result_1.clone();
lookup(ctx.clone(), n.clone(), lookup_result_0.clone()) <-- lookup_input(ctx, n), let _tmp_0 = destruct_Bind(ctx.clone()), if !_tmp_0.is_none(), let (n1, ty, rest)=_tmp_0.unwrap(), if *n.clone() != n1.clone(), lookup(rest, n, lookup_call_1), let lookup_result_0 = lookup_call_1.clone();
relation eqType_input(Type, Type);
eqType_input(ty1_0.clone(), ty2_0.clone()) <-- typeOf_input(ctx, exp), let _tmp_0 = destruct_App(exp.clone()), if !_tmp_0.is_none(), let (fun, arg)=_tmp_0.unwrap(), typeOf(ctx, fun, typeOf_call_4), let _tmp_1 = destruct_Some_Type(typeOf_call_4.clone()), if !_tmp_1.is_none(), let funty=_tmp_1.unwrap(), let _tmp_2 = destruct_TFun(funty.clone()), if !_tmp_2.is_none(), let (ty2_0, ty2)=_tmp_2.unwrap(), typeOf(ctx, arg, typeOf_call_5), let _tmp_3 = destruct_Some_Type(typeOf_call_5.clone()), if !_tmp_3.is_none(), let ty1_0=_tmp_3.unwrap();
eqType_input(ty1_0.clone(), ty2_0.clone()) <-- eqType_input(ty1, ty2), let _tmp_0 = destruct_TFun(ty1.clone()), if !_tmp_0.is_none(), let (ty1_0, fty2)=_tmp_0.unwrap(), let _tmp_1 = destruct_TFun(ty2.clone()), if !_tmp_1.is_none(), let (ty2_0, ofty2)=_tmp_1.unwrap();
eqType_input(ty1_0.clone(), ty2_0.clone()) <-- eqType_input(ty1, ty2), let _tmp_0 = destruct_TFun(ty1.clone()), if !_tmp_0.is_none(), let (fty1, ty1_0)=_tmp_0.unwrap(), let _tmp_1 = destruct_TFun(ty2.clone()), if !_tmp_1.is_none(), let (ofty1, ty2_0)=_tmp_1.unwrap(), eqType(fty1, ofty1, eqType_call_1);
relation typeOf_input(Ctx, Exp);
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- program(exp_0), let ctx_0 = Ctx::Empty();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx_0, exp), let _tmp_0 = destruct_App(exp.clone()), if !_tmp_0.is_none(), let (exp_0, arg)=_tmp_0.unwrap();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx_0, exp), let _tmp_0 = destruct_App(exp.clone()), if !_tmp_0.is_none(), let (fun, exp_0)=_tmp_0.unwrap(), typeOf(ctx_0, fun, typeOf_call_4), let _tmp_1 = destruct_Some_Type(typeOf_call_4.clone()), if !_tmp_1.is_none(), let funty=_tmp_1.unwrap(), let _tmp_2 = destruct_TFun(funty.clone()), if !_tmp_2.is_none(), let (ty1, ty2)=_tmp_2.unwrap();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx, exp), let _tmp_0 = destruct_Let(exp.clone()), if !_tmp_0.is_none(), let (n, bound, exp_0)=_tmp_0.unwrap(), typeOf(ctx, bound, typeOf_call_6), let _tmp_1 = destruct_Some_Type(typeOf_call_6.clone()), if !_tmp_1.is_none(), let boundty=_tmp_1.unwrap(), let ctx_0 = Ctx::Bind((n.clone()).to_string(),boundty.clone(),Box::new(ctx.clone()));
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx_0, exp), let _tmp_0 = destruct_Add(exp.clone()), if !_tmp_0.is_none(), let (exp_0, r)=_tmp_0.unwrap();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- extendCtx_input(ctx_0, bindings), let _tmp_0 = destruct_Cons(bindings.clone()), if !_tmp_0.is_none(), let (name, exp_0, rest)=_tmp_0.unwrap();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx, exp), let _tmp_0 = destruct_LetStar(exp.clone()), if !_tmp_0.is_none(), let (bindings, exp_0)=_tmp_0.unwrap(), extendCtx(ctx, bindings, extendCtx_call_0), let _tmp_1 = destruct_Some_Ctx(extendCtx_call_0.clone()), if !_tmp_1.is_none(), let ctx_0=_tmp_1.unwrap();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx, exp), let _tmp_0 = destruct_Lam(exp.clone()), if !_tmp_0.is_none(), let (n, ty, exp_0)=_tmp_0.unwrap(), let ctx_0 = Ctx::Bind((n.clone()).to_string(),ty.clone(),Box::new(ctx.clone()));
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx_0, exp), let _tmp_0 = destruct_Add(exp.clone()), if !_tmp_0.is_none(), let (l, exp_0)=_tmp_0.unwrap(), typeOf(ctx_0, l, typeOf_call_1), let _tmp_1 = destruct_Some_Type(typeOf_call_1.clone()), if !_tmp_1.is_none(), let lty=_tmp_1.unwrap(), let _tmp_2 = destruct_TInt(lty.clone()), if !_tmp_2.is_none(), let ()=_tmp_2.unwrap();
typeOf_input(ctx_0.clone(), exp_0.clone()) <-- typeOf_input(ctx_0, exp), let _tmp_0 = destruct_Let(exp.clone()), if !_tmp_0.is_none(), let (n, exp_0, body)=_tmp_0.unwrap();

  relation main(Option_Type);
  main(main_result_0.clone()) <--
    program(program_call_0),
    typeOf(Ctx::Empty(), program_call_0, main_result_0);
  
  relation extendCtx(Ctx, BindingList, Option_Ctx);
  extendCtx(ctx.clone(), bindings.clone(), extendCtx_result_0.clone()) <--
    extendCtx_input(ctx, bindings),
    let _tmp_0 = destruct_Nil(bindings.clone()),
    if !_tmp_0.is_none(), let ()=_tmp_0.unwrap(),
    let extendCtx_result_0 = Option_Ctx::Some_Ctx(ctx.clone());
  extendCtx(ctx.clone(), bindings.clone(), extendCtx_result_0.clone()) <--
    extendCtx_input(ctx, bindings),
    let _tmp_0 = destruct_Cons(bindings.clone()),
    if !_tmp_0.is_none(), let (name, bound, rest)=_tmp_0.unwrap(),
    typeOf(ctx, bound, typeOf_call_9),
    let _tmp_1 = destruct_None_Type(typeOf_call_9.clone()),
    if !_tmp_1.is_none(), let ()=_tmp_1.unwrap(),
    let match_result_12 = Option_Ctx::None_Ctx(),
    let extendCtx_result_0 = match_result_12.clone();
  extendCtx(ctx.clone(), bindings.clone(), extendCtx_result_0.clone()) <--
    extendCtx_input(ctx, bindings),
    let _tmp_0 = destruct_Cons(bindings.clone()),
    if !_tmp_0.is_none(), let (name, bound, rest)=_tmp_0.unwrap(),
    typeOf(ctx, bound, typeOf_call_9),
    let _tmp_1 = destruct_Some_Type(typeOf_call_9.clone()),
    if !_tmp_1.is_none(), let ty=_tmp_1.unwrap(),
    extendCtx(Ctx::Bind((name.clone()).to_string(),ty.clone(),Box::new(ctx.clone())), rest, extendCtx_call_1),
    let extendCtx_result_0 = extendCtx_call_1.clone();

  relation extendCtx_input(Ctx, BindingList);
  extendCtx_input(ctx_0.clone(), bindings_0.clone()) <--
    typeOf_input(ctx_0, exp),
    let _tmp_0 = destruct_LetStar(exp.clone()),
    if !_tmp_0.is_none(), let (bindings_0, body)=_tmp_0.unwrap();
  extendCtx_input(ctx_0.clone(), bindings_0.clone()) <--
    extendCtx_input(ctx, bindings),
    let _tmp_0 = destruct_Cons(bindings.clone()),
    if !_tmp_0.is_none(), let (name, bound, bindings_0)=_tmp_0.unwrap(),
    typeOf(ctx, bound, typeOf_call_9),
    let _tmp_1 = destruct_Some_Type(typeOf_call_9.clone()),
    if !_tmp_1.is_none(), let ty=_tmp_1.unwrap(),
    let ctx_0 = Ctx::Bind((name.clone()).to_string(),ty.clone(),Box::new(ctx.clone()));
  
  relation program(Exp);
  program(program_result_0.clone()) <--
    let program_result_0 = Exp::Lam((<&str as Into<String>>::into(r###"m"###)).to_string(),Type::TFun(Box::new(Type::TFun(Box::new(Type::TInt()),Box::new(Type::TInt()))),Box::new(Type::TFun(Box::new(Type::TInt()),Box::new(Type::TInt())))),Box::new(Exp::Lam((<&str as Into<String>>::into(r###"n"###)).to_string(),Type::TFun(Box::new(Type::TFun(Box::new(Type::TInt()),Box::new(Type::TInt()))),Box::new(Type::TFun(Box::new(Type::TInt()),Box::new(Type::TInt())))),Box::new(Exp::Lam((<&str as Into<String>>::into(r###"f"###)).to_string(),Type::TFun(Box::new(Type::TInt()),Box::new(Type::TInt())),Box::new(Exp::Lam((<&str as Into<String>>::into(r###"x"###)).to_string(),Type::TInt(),Box::new(Exp::App(Box::new(Exp::App(Box::new(Exp::Var((<&str as Into<String>>::into(r###"m"###)).to_string())),Box::new(Exp::Var((<&str as Into<String>>::into(r###"f"###)).to_string())))),Box::new(Exp::App(Box::new(Exp::App(Box::new(Exp::Var((<&str as Into<String>>::into(r###"n"###)).to_string())),Box::new(Exp::Var((<&str as Into<String>>::into(r###"f"###)).to_string())))),Box::new(Exp::Var((<&str as Into<String>>::into(r###"x"###)).to_string())))))))))))));
}

#[test]
fn inca_codegen() {
   let mut prog = AscentProgram::default();

   use std::time::Instant;
   let now = Instant::now();
   prog.run();
   let elapsed = now.elapsed().as_millis();
   println!("{}", elapsed);

   println!(
      "{{\"name\": \"lookup_input\", \"size\": 2, \"elements\": {:?}}}",
      &prog.lookup_input
   );
   println!(
      "{{\"name\": \"typeOf\", \"size\": 3, \"elements\": {:?}}}",
      &prog.typeOf
   );
   println!(
      "{{\"name\": \"eqType\", \"size\": 3, \"elements\": {:?}}}",
      &prog.eqType
   );
   println!(
      "{{\"name\": \"lookup\", \"size\": 3, \"elements\": {:?}}}",
      &prog.lookup
   );
   println!(
      "{{\"name\": \"eqType_input\", \"size\": 2, \"elements\": {:?}}}",
      &prog.eqType_input
   );
   println!(
      "{{\"name\": \"typeOf_input\", \"size\": 2, \"elements\": {:?}}}",
      &prog.typeOf_input
   );
   println!(
      "{{\"name\": \"extendCtx\", \"size\": 3, \"elements\": {:?}}}",
      &prog.extendCtx
   );
   println!(
      "{{\"name\": \"extendCtx_input\", \"size\": 2, \"elements\": {:?}}}",
      &prog.extendCtx_input
   );
   println!(
      "{{\"name\": \"program\", \"size\": 1, \"elements\": {:?}}}",
      &prog.program
   );
}
