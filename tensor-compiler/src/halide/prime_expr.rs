use std::fmt::Display;

use super::{
    exprs::*,
    traits::{ Accepter, AccepterMut, AccepterMutate, IRMutVisitor, IRMutateVisitor, IRVisitor },
    variable::Variable,
};

#[derive(Clone, PartialEq, Hash, Eq, Debug)]
pub enum PrimeExpr {
    Int(Int),
    Float(Float),
    UInt(UInt),
    Str(Str),
    Variable(Variable),
    Reduce(Reduce),
    Cast(Cast),
    Add(Add),
    Sub(Sub),
    Mul(Mul),
    Div(Div),
    Mod(Mod),
    Min(Min),
    Max(Max),
    Eq(Eq),
    Ne(Ne),
    Lt(Lt),
    Le(Le),
    Gt(Gt),
    Ge(Ge),
    And(And),
    Or(Or),
    Xor(Xor),
    Not(Not),
    Call(Call),
    Select(Select),
    Let(Let),
    Load(Load),
    None,
}

#[derive(Clone, Copy, PartialEq, Hash, Eq, PartialOrd, Ord)]
pub enum PrimeType {
    Int,
    Float,
    UInt,
    Str,
    Variable,
    Reduce,
    Cast,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Min,
    Max,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Xor,
    Not,
    Call,
    Select,
    Let,
    Load,
    None,
}

macro_rules! cast_expr {
    ($fn_name:ident, $t:ident) => {
        pub fn $fn_name(&self) -> Option<&$t> {
            match self {
                PrimeExpr::$t(e) => Some(e),
                _ => None,
            }
        }
    };
}

impl PrimeExpr {
    pub const fn is_none(&self) -> bool {
        matches!(self, PrimeExpr::None)
    }

    pub const fn type_info(&self) -> PrimeType {
        match self {
            PrimeExpr::Int(_) => PrimeType::Int,
            PrimeExpr::Float(_) => PrimeType::Float,
            PrimeExpr::UInt(_) => PrimeType::UInt,
            PrimeExpr::Str(_) => PrimeType::Str,
            PrimeExpr::Variable(_) => PrimeType::Variable,
            PrimeExpr::Cast(_) => PrimeType::Cast,
            PrimeExpr::Add(_) => PrimeType::Add,
            PrimeExpr::Sub(_) => PrimeType::Sub,
            PrimeExpr::Mul(_) => PrimeType::Mul,
            PrimeExpr::Div(_) => PrimeType::Div,
            PrimeExpr::Mod(_) => PrimeType::Mod,
            PrimeExpr::Min(_) => PrimeType::Min,
            PrimeExpr::Max(_) => PrimeType::Max,
            PrimeExpr::Eq(_) => PrimeType::Eq,
            PrimeExpr::Ne(_) => PrimeType::Ne,
            PrimeExpr::Lt(_) => PrimeType::Lt,
            PrimeExpr::Le(_) => PrimeType::Le,
            PrimeExpr::Gt(_) => PrimeType::Gt,
            PrimeExpr::Ge(_) => PrimeType::Ge,
            PrimeExpr::And(_) => PrimeType::And,
            PrimeExpr::Xor(_) => PrimeType::Xor,
            PrimeExpr::Or(_) => PrimeType::Or,
            PrimeExpr::Not(_) => PrimeType::Not,
            PrimeExpr::Call(_) => PrimeType::Call,
            PrimeExpr::Select(_) => PrimeType::Select,
            PrimeExpr::Let(_) => PrimeType::Let,
            PrimeExpr::Load(_) => PrimeType::Load,
            PrimeExpr::Reduce(_) => PrimeType::Reduce,
            PrimeExpr::None => PrimeType::None,
        }
    }

    const fn precedence(&self) -> i32 {
        match self {
            PrimeExpr::Add(_) | PrimeExpr::Sub(_) => 1,
            PrimeExpr::Mul(_) | PrimeExpr::Div(_) | PrimeExpr::Mod(_) => 2,
            _ => 3,
        }
    }

    fn print(&self, parent_prec: i32) -> String {
        let prec = self.precedence();
        let s = match self {
            PrimeExpr::Int(a) => a.to_string(),
            PrimeExpr::Float(a) => a.to_string(),
            PrimeExpr::UInt(a) => a.to_string(),
            PrimeExpr::Str(a) => a.to_string(),
            PrimeExpr::Variable(a) => a.to_string(),
            PrimeExpr::Cast(a) => a.to_string(),
            PrimeExpr::Add(a) => format!("{} + {}", a.e1().print(prec), a.e2().print(prec + 1)),
            PrimeExpr::Sub(a) => format!("{} - {}", a.e1().print(prec), a.e2().print(prec + 1)),
            PrimeExpr::Mul(a) => format!("{} * {}", a.e1().print(prec), a.e2().print(prec + 1)),
            PrimeExpr::Div(a) => format!("{} / {}", a.e1().print(prec), a.e2().print(prec + 1)),
            PrimeExpr::Mod(a) => format!("{} % {}", a.e1().print(prec), a.e2().print(prec + 1)),
            PrimeExpr::Min(a) => a.to_string(),
            PrimeExpr::Max(a) => a.to_string(),
            PrimeExpr::Eq(a) => a.to_string(),
            PrimeExpr::Ne(a) => a.to_string(),
            PrimeExpr::Lt(a) => a.to_string(),
            PrimeExpr::Le(a) => a.to_string(),
            PrimeExpr::Gt(a) => a.to_string(),
            PrimeExpr::Ge(a) => a.to_string(),
            PrimeExpr::And(a) => a.to_string(),
            PrimeExpr::Xor(a) => a.to_string(),
            PrimeExpr::Or(a) => a.to_string(),
            PrimeExpr::Not(a) => a.to_string(),
            PrimeExpr::Call(a) => a.to_string(),
            PrimeExpr::Select(a) => a.to_string(),
            PrimeExpr::Let(a) => a.to_string(),
            PrimeExpr::Load(a) => a.to_string(),
            PrimeExpr::Reduce(a) => a.to_string(),
            PrimeExpr::None => "".to_string(),
        };
        if prec < parent_prec {
            format!("({})", s)
        } else {
            s
        }
    }

    cast_expr!(to_variable, Variable);
    cast_expr!(to_add, Add);
    cast_expr!(to_sub, Sub);
    cast_expr!(to_mul, Mul);
    cast_expr!(to_div, Div);
    cast_expr!(to_mod, Mod);
    cast_expr!(to_min, Min);
    cast_expr!(to_max, Max);
    cast_expr!(to_eq, Eq);
    cast_expr!(to_ne, Ne);
    cast_expr!(to_lt, Lt);
    cast_expr!(to_le, Le);
    cast_expr!(to_gt, Gt);
    cast_expr!(to_ge, Ge);
    cast_expr!(to_and, And);
    cast_expr!(to_or, Or);
    cast_expr!(to_not, Not);
    cast_expr!(to_call, Call);
    cast_expr!(to_select, Select);
    cast_expr!(to_let, Let);
    cast_expr!(to_load, Load);
    cast_expr!(to_int, Int);
    cast_expr!(to_float, Float);
    cast_expr!(to_uint, UInt);
    cast_expr!(to_str, Str);
    cast_expr!(to_cast, Cast);
}

impl Into<PrimeExpr> for &PrimeExpr {
    fn into(self) -> PrimeExpr {
        self.clone()
    }
}

impl Into<PrimeExpr> for &&PrimeExpr {
    fn into(self) -> PrimeExpr {
        (*self).clone()
    }
}

impl Display for PrimeExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.print(0))
    }
}

impl Accepter for PrimeExpr {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_expr(self);
    }
}

impl AccepterMut for PrimeExpr {
    fn accept_mut<V: IRMutVisitor>(&self, visitor: &mut V) {
        visitor.visit_expr(self);
    }
}

impl AccepterMutate for PrimeExpr {
    fn accept_mutate<V: IRMutateVisitor>(&self, visitor: &mut V) {
        visitor.visit_expr(self);
    }
}

impl std::ops::Add for PrimeExpr {
    type Output = PrimeExpr;

    fn add(self, rhs: PrimeExpr) -> Self::Output {
        match (&self, &rhs) {
            (PrimeExpr::Int(i1), PrimeExpr::Int(i2)) => PrimeExpr::Int(i1 + i2),
            (PrimeExpr::Float(f1), PrimeExpr::Float(f2)) => PrimeExpr::Float(Float::new(f1.value() + f2.value())),
            (PrimeExpr::Int(i), PrimeExpr::Float(f)) =>
                PrimeExpr::Float(Float::new((i.value() as f64) + f.value())),
            (PrimeExpr::Float(f), PrimeExpr::Int(i)) =>
                PrimeExpr::Float(Float::new(f.value() + (i.value() as f64))),
            (PrimeExpr::UInt(u1), PrimeExpr::UInt(u2)) => PrimeExpr::UInt(u1 + u2),
            (PrimeExpr::Mul(m1), PrimeExpr::Mul(m2)) => PrimeExpr::Add(Add::new(m1.into(), m2.into())),
            (PrimeExpr::Add(a1), PrimeExpr::Add(a2)) => PrimeExpr::Add(Add::new(a1.into(), a2.into())),
            (PrimeExpr::Sub(s1), PrimeExpr::Sub(s2)) => PrimeExpr::Add(Add::new(s1.into(), s2.into())),
            (PrimeExpr::Div(d1), PrimeExpr::Div(d2)) => PrimeExpr::Add(Add::new(d1.into(), d2.into())),
            (PrimeExpr::Mod(m1), PrimeExpr::Mod(m2)) => PrimeExpr::Add(Add::new(m1.into(), m2.into())),
            (PrimeExpr::Add(a), PrimeExpr::Mul(m)) => PrimeExpr::Add(Add::new(a.into(), m.into())),
            (PrimeExpr::Add(a), PrimeExpr::Sub(s)) => PrimeExpr::Add(Add::new(a.into(), s.into())),
            (PrimeExpr::Add(a), PrimeExpr::Div(d)) => PrimeExpr::Add(Add::new(a.into(), d.into())),
            (PrimeExpr::Add(a), PrimeExpr::Mod(m)) => PrimeExpr::Add(Add::new(a.into(), m.into())),
            (PrimeExpr::Mul(m), PrimeExpr::Add(a)) => PrimeExpr::Add(Add::new(m.into(), a.into())),
            (PrimeExpr::Sub(s), PrimeExpr::Add(a)) => PrimeExpr::Add(Add::new(s.into(), a.into())),
            (PrimeExpr::Div(d), PrimeExpr::Add(a)) => PrimeExpr::Add(Add::new(d.into(), a.into())),
            (PrimeExpr::Mod(m), PrimeExpr::Add(a)) => PrimeExpr::Add(Add::new(m.into(), a.into())),
            (PrimeExpr::Add(a), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(a.into(), i.into())),
            (PrimeExpr::Sub(s), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(s.into(), i.into())),
            (PrimeExpr::Mul(m), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(m.into(), i.into())),
            (PrimeExpr::Div(d), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(d.into(), i.into())),
            (PrimeExpr::Mod(m), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(m.into(), i.into())),
            (PrimeExpr::Variable(v), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(v.into(), i.into())),
            (PrimeExpr::Int(i), PrimeExpr::Variable(v)) => PrimeExpr::Add(Add::new(i.into(), v.into())),
            (PrimeExpr::Variable(v), PrimeExpr::Variable(v2)) => PrimeExpr::Add(Add::new(v.into(), v2.into())),
            (PrimeExpr::Variable(v), PrimeExpr::Add(i)) => PrimeExpr::Add(Add::new(v.into(), i.into())),
            (PrimeExpr::Add(i), PrimeExpr::Variable(v)) => PrimeExpr::Add(Add::new(i.into(), v.into())),
            (PrimeExpr::Load(l), PrimeExpr::Int(i)) => PrimeExpr::Add(Add::new(l.into(), i.into())),
            (PrimeExpr::Int(i), PrimeExpr::Load(l)) => PrimeExpr::Add(Add::new(i.into(), l.into())),
            (PrimeExpr::Load(l), PrimeExpr::Load(l2)) => PrimeExpr::Add(Add::new(l.into(), l2.into())),
            (PrimeExpr::Load(l), PrimeExpr::Add(i)) => PrimeExpr::Add(Add::new(l.into(), i.into())),
            (PrimeExpr::Add(i), PrimeExpr::Load(l)) => PrimeExpr::Add(Add::new(i.into(), l.into())),
            _ => panic!("{}", &format!("Failed to add {} and {}", self, rhs)),
        }
    }
}
