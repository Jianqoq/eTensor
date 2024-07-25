use std::{ fmt::Display, sync::Arc };

use tensor_types::dtype::Dtype;

use crate::iter_var::IterVar ;

use super::{ prime_expr::PrimeExpr, traits::{ Accepter, IRVisitor }, variable::Variable };
#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Int {
    value: i64,
    dtype: Dtype,
}

impl Int {
    pub fn make(dtype: Dtype, mut value: i64) -> Self {
        value = value << (64 - dtype.bits());
        value = value >> (64 - dtype.bits());
        Int { value, dtype }
    }

    pub fn value(&self) -> i64 {
        self.value
    }

    pub fn dtype(&self) -> &Dtype {
        &self.dtype
    }
}

impl Accepter for Int {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_int(self);
    }
}

impl Display for Int {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Into<PrimeExpr> for Int {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Int(self)
    }
}

impl Into<PrimeExpr> for &Int {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Int(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct UInt {
    value: u64,
    dtype: Dtype,
}

impl UInt {
    pub fn make(dtype: Dtype, mut value: u64) -> Self {
        value = value << (64 - dtype.bits());
        value = value >> (64 - dtype.bits());
        UInt { value, dtype }
    }

    pub fn value(&self) -> u64 {
        self.value
    }

    pub fn dtype(&self) -> &Dtype {
        &self.dtype
    }
}

impl Accepter for UInt {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_uint(self);
    }
}

impl Display for UInt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Into<PrimeExpr> for UInt {
    fn into(self) -> PrimeExpr {
        PrimeExpr::UInt(self)
    }
}

impl Into<PrimeExpr> for &UInt {
    fn into(self) -> PrimeExpr {
        PrimeExpr::UInt(self.clone())
    }
}

macro_rules! impl_binop {
    ($lhs:ident, $rhs:ident, $res:ident, $std_op:ident, $std_op_name:ident, $op:tt) => {
        impl std::ops::$std_op for $lhs {
            type Output = $res;

            fn $std_op_name(self, rhs: $rhs) -> Self::Output {
                $res::make(self.dtype().clone(), self.value $op rhs.value)
            }
        }
        impl std::ops::$std_op for &$lhs {
            type Output = $res;

            fn $std_op_name(self, rhs: &$rhs) -> Self::Output {
                $res::make(self.dtype().clone(), self.value $op rhs.value)
            }
        }

        impl std::ops::$std_op<$rhs> for &$lhs {
            type Output = $res;

            fn $std_op_name(self, rhs: $rhs) -> Self::Output {
                $res::make(self.dtype().clone(), self.value $op rhs.value)
            }
        }

        impl std::ops::$std_op<&$rhs> for $lhs {
            type Output = $res;

            fn $std_op_name(self, rhs: &$rhs) -> Self::Output {
                $res::make(self.dtype().clone(), self.value $op rhs.value)
            }
        }
    };
}

impl_binop!(Int, Int, Int, Add, add, +);
impl_binop!(Int, Int, Int, Sub, sub, -);
impl_binop!(Int, Int, Int, Mul, mul, *);
impl_binop!(Int, Int, Int, Div, div, /);
impl_binop!(Int, Int, Int, Rem, rem, %);

impl_binop!(UInt, UInt, UInt, Add, add, +);
impl_binop!(UInt, UInt, UInt, Sub, sub, -);
impl_binop!(UInt, UInt, UInt, Mul, mul, *);
impl_binop!(UInt, UInt, UInt, Div, div, /);
impl_binop!(UInt, UInt, UInt, Rem, rem, %);

#[derive(Clone, PartialEq, Debug)]
pub struct Float {
    value: f64,
    dtype: Dtype,
}

impl std::cmp::Eq for Float {}

impl std::hash::Hash for Float {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        f64::to_be_bytes(self.value).hash(state);
    }
}

impl Float {
    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn make(dtype: Dtype, value: f64) -> Self {
        Float { value, dtype }
    }
    pub fn dtype(&self) -> &Dtype {
        &self.dtype
    }
}

impl Accepter for Float {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_float(self);
    }
}

impl Display for Float {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Into<PrimeExpr> for Float {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Float(self)
    }
}

impl Into<PrimeExpr> for &Float {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Float(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Str {
    value: Arc<String>,
}

impl Str {
    pub fn new(value: String) -> Self {
        Str {
            value: value.into(),
        }
    }

    pub fn make(value: &str) -> Self {
        Str {
            value: value.to_string().into(),
        }
    }

    pub fn value(&self) -> &str {
        &self.value
    }
}

impl Accepter for Str {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_str(self);
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Into<PrimeExpr> for Str {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Str(self)
    }
}

impl Into<PrimeExpr> for &Str {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Str(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Cast {
    expr: Arc<PrimeExpr>,
    dtype: Dtype,
}

impl Cast {
    pub fn new(expr: Arc<PrimeExpr>, dtype: Dtype) -> Self {
        Cast { expr, dtype }
    }

    pub fn make<T: Into<PrimeExpr>>(expr: T, dtype: Dtype) -> Self {
        Cast {
            expr: expr.into().into(),
            dtype,
        }
    }

    pub fn expr(&self) -> &PrimeExpr {
        &self.expr
    }

    pub fn expr_(&self) -> &Arc<PrimeExpr> {
        &self.expr
    }

    pub fn dtype(&self) -> &Dtype {
        &self.dtype
    }
}

impl Accepter for Cast {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_cast(self);
    }
}

impl Display for Cast {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} as {})", self.expr, self.dtype)
    }
}

impl Into<PrimeExpr> for Cast {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Cast(self)
    }
}

impl Into<PrimeExpr> for &Cast {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Cast(self.clone())
    }
}

#[derive(Clone, Hash, Eq, Debug)]
pub struct Add {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl PartialEq for Add {
    fn eq(&self, other: &Self) -> bool {
        (self.e1 == other.e1 && self.e2 == other.e2) || (self.e1 == other.e2 && self.e2 == other.e1)
    }
}

impl Add {
    pub fn new(e1: PrimeExpr, e2: PrimeExpr) -> Self {
        Add {
            e1: e1.into(),
            e2: e2.into(),
        }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Add {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
    pub fn set_e1<A: Into<PrimeExpr>>(&mut self, e1: A) {
        self.e1 = e1.into().into();
    }

    pub fn set_e2<A: Into<PrimeExpr>>(&mut self, e2: A) {
        self.e2 = e2.into().into();
    }
}

impl Accepter for Add {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_add(self);
    }
}

impl Display for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} + {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Add {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Add(self)
    }
}

impl Into<PrimeExpr> for &Add {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Add(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Sub {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Sub {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_sub(self);
    }
}

impl Sub {
    pub fn new<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Sub { e1: e1.into().into(), e2: e2.into().into() }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Sub {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} - {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Sub {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Sub(self)
    }
}

impl Into<PrimeExpr> for &Sub {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Sub(self.clone())
    }
}

#[derive(Clone, Hash, Eq, Debug)]
pub struct Mul {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl PartialEq for Mul {
    fn eq(&self, other: &Self) -> bool {
        (self.e1 == other.e1 && self.e2 == other.e2) || (self.e1 == other.e2 && self.e2 == other.e1)
    }
}

impl Accepter for Mul {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_mul(self);
    }
}

impl Mul {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Mul { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Mul {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} * {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Mul {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Mul(self)
    }
}

impl Into<PrimeExpr> for &Mul {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Mul(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Div {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Div {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_div(self);
    }
}

impl Div {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Div { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Div {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} / {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Div {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Div(self)
    }
}

impl Into<PrimeExpr> for &Div {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Div(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Neg {
    e: Arc<PrimeExpr>,
}

impl Accepter for Neg {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_neg(self);
    }
}

impl Neg {
    pub fn new(e: Arc<PrimeExpr>) -> Self {
        Neg { e }
    }

    pub fn make<T: Into<PrimeExpr>>(e: T) -> Self {
        Neg { e: e.into().into() }
    }

    pub fn e(&self) -> &PrimeExpr {
        &self.e
    }

    pub fn e_(&self) -> &Arc<PrimeExpr> {
        &self.e
    }
}

impl Display for Neg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "-{}", self.e)
    }
}

impl Into<PrimeExpr> for Neg {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Neg(self)
    }
}

impl Into<PrimeExpr> for &Neg {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Neg(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct FloorDiv {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for FloorDiv {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_floor_div(self);
    }
}

impl FloorDiv {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        FloorDiv { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        FloorDiv {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for FloorDiv {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} // {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for FloorDiv {
    fn into(self) -> PrimeExpr {
        PrimeExpr::FloorDiv(self)
    }
}

impl Into<PrimeExpr> for &FloorDiv {
    fn into(self) -> PrimeExpr {
        PrimeExpr::FloorDiv(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Gt {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Gt {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_gt(self);
    }
}

impl Gt {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Gt { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Gt {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Gt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} > {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Gt {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Gt(self)
    }
}

impl Into<PrimeExpr> for &Gt {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Gt(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Ge {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Ge {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_ge(self);
    }
}

impl Ge {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Ge { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        Ge {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Ge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} >= {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Ge {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Ge(self)
    }
}

impl Into<PrimeExpr> for &Ge {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Ge(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct BitAnd {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for BitAnd {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_and(self);
    }
}

impl BitAnd {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        BitAnd { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        BitAnd {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for BitAnd {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} && {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for BitAnd {
    fn into(self) -> PrimeExpr {
        PrimeExpr::BitAnd(self)
    }
}

impl Into<PrimeExpr> for &BitAnd {
    fn into(self) -> PrimeExpr {
        PrimeExpr::BitAnd(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct BitOr {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for BitOr {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_or(self);
    }
}

impl BitOr {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        BitOr { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        BitOr {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for BitOr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} || {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for BitOr {
    fn into(self) -> PrimeExpr {
        PrimeExpr::BitOr(self)
    }
}

impl Into<PrimeExpr> for &BitOr {
    fn into(self) -> PrimeExpr {
        PrimeExpr::BitOr(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct BitXor {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for BitXor {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_xor(self);
    }
}

impl BitXor {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        BitXor { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        BitXor {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for BitXor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} ^ {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for BitXor {
    fn into(self) -> PrimeExpr {
        PrimeExpr::BitXor(self)
    }
}

impl Into<PrimeExpr> for &BitXor {
    fn into(self) -> PrimeExpr {
        PrimeExpr::BitXor(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Not {
    e: Arc<PrimeExpr>,
}

impl Accepter for Not {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_not(self);
    }
}

impl Not {
    pub fn new(e: Arc<PrimeExpr>) -> Self {
        Not { e }
    }

    pub fn make<T: Into<PrimeExpr>>(e: T) -> Self {
        Not { e: e.into().into() }
    }

    pub fn e(&self) -> &PrimeExpr {
        &self.e
    }

    pub fn e_(&self) -> &Arc<PrimeExpr> {
        &self.e
    }
}

impl Display for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(!{})", self.e)
    }
}

impl Into<PrimeExpr> for Not {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Not(self)
    }
}

impl Into<PrimeExpr> for &Not {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Not(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Shl {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Shl {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_shl(self);
    }
}

impl Shl {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Shl { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        Shl {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Shl {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} << {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Shl {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Shl(self)
    }
}

impl Into<PrimeExpr> for &Shl {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Shl(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Shr {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Shr {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_shr(self);
    }
}

impl Shr {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Shr { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        Shr {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Shr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} >> {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Shr {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Shr(self)
    }
}

impl Into<PrimeExpr> for &Shr {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Shr(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Call {
    name: Arc<String>,
    args: Vec<Arc<PrimeExpr>>,
}

impl Accepter for Call {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_call(self);
    }
}

impl Call {
    pub fn new(name: &str, args: Vec<Arc<PrimeExpr>>) -> Self {
        Call {
            name: name.to_string().into(),
            args,
        }
    }

    pub fn make<T: Into<PrimeExpr> + Clone>(name: &str, args: &[T]) -> Self {
        Call {
            name: name.to_string().into(),
            args: args
                .iter()
                .map(|e| e.clone().into().into())
                .collect(),
        }
    }

    pub fn name(&self) -> &String {
        &self.name
    }

    pub fn args(&self) -> &Vec<Arc<PrimeExpr>> {
        &self.args
    }

    pub fn args_(&self) -> &Vec<Arc<PrimeExpr>> {
        &self.args
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.name,
            self.args
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

impl Into<PrimeExpr> for Call {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Call(self)
    }
}

impl Into<PrimeExpr> for &Call {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Call(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Select {
    cond: Arc<PrimeExpr>,
    true_expr: Arc<PrimeExpr>,
    false_expr: Arc<PrimeExpr>,
}

impl Accepter for Select {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_select(self);
    }
}

impl Select {
    pub fn new(
        cond: Arc<PrimeExpr>,
        true_expr: Arc<PrimeExpr>,
        false_expr: Arc<PrimeExpr>
    ) -> Self {
        Select {
            cond,
            true_expr,
            false_expr,
        }
    }

    pub fn make<T: Into<PrimeExpr>>(cond: T, true_expr: T, false_expr: T) -> Self {
        Select {
            cond: cond.into().into(),
            true_expr: true_expr.into().into(),
            false_expr: false_expr.into().into(),
        }
    }

    pub fn cond(&self) -> &PrimeExpr {
        &self.cond
    }

    pub fn true_expr(&self) -> &PrimeExpr {
        &self.true_expr
    }

    pub fn false_expr(&self) -> &PrimeExpr {
        &self.false_expr
    }

    pub fn cond_(&self) -> &Arc<PrimeExpr> {
        &self.cond
    }

    pub fn true_expr_(&self) -> &Arc<PrimeExpr> {
        &self.true_expr
    }

    pub fn false_expr_(&self) -> &Arc<PrimeExpr> {
        &self.false_expr
    }
}

impl Display for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} ? {} : {})", self.cond, self.true_expr, self.false_expr)
    }
}

impl Into<PrimeExpr> for Select {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Select(self)
    }
}

impl Into<PrimeExpr> for &Select {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Select(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Load {
    pub(crate) name: Variable,
    pub(crate) indices: Arc<PrimeExpr>,
}

impl Accepter for Load {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_load(self);
    }
}

impl Load {
    pub fn make_from_strides(name: &Variable, indices: &[Variable], strides: &[i64]) -> Self {
        if indices.len() != strides.len() {
            panic!(
                "Indices and strides must have the same length, got {:?} and {:?}",
                indices,
                strides
            );
        }
        let indices = indices
            .iter()
            .zip(strides.iter())
            .map(|(v, s)| v.clone() * Int::make(Dtype::I64, *s))
            .reduce(|a, b| a + b)
            .unwrap();
        Load {
            name: name.into(),
            indices: indices.into(),
        }
    }

    pub fn make<A: Into<Variable>, B: Into<PrimeExpr>>(name: A, indices: B) -> Self {
        Load {
            name: name.into(),
            indices: indices.into().into(),
        }
    }

    pub fn name(&self) -> &Variable {
        &self.name
    }

    pub fn indices(&self) -> &PrimeExpr {
        &self.indices
    }

    pub fn name_(&self) -> &Variable {
        &self.name
    }

    pub fn indices_(&self) -> &Arc<PrimeExpr> {
        &self.indices
    }
}

impl Display for Load {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}[{}]", self.name, self.indices)
    }
}

impl Into<PrimeExpr> for Load {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Load(self)
    }
}

impl Into<PrimeExpr> for &Load {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Load(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Let {
    name: Variable,
    value: Arc<PrimeExpr>,
    body: Arc<PrimeExpr>,
}

impl Accepter for Let {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_let(self);
    }
}

impl Let {
    pub fn new(name: Variable, value: Arc<PrimeExpr>, body: Arc<PrimeExpr>) -> Self {
        Let { name, value, body }
    }

    pub fn make<T: Into<PrimeExpr>, B: Into<PrimeExpr>>(
        name: &Variable,
        value: T,
        body: B
    ) -> Self {
        Let {
            name: name.clone(),
            value: value.into().into(),
            body: body.into().into(),
        }
    }

    pub fn name(&self) -> &Variable {
        &self.name
    }

    pub fn value(&self) -> &PrimeExpr {
        &self.value
    }

    pub fn value_(&self) -> &Arc<PrimeExpr> {
        &self.value
    }

    pub fn body(&self) -> &PrimeExpr {
        &self.body
    }

    pub fn body_(&self) -> &Arc<PrimeExpr> {
        &self.body
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "let {} = {};", self.name, self.value)
    }
}

impl Into<PrimeExpr> for Let {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Let(self)
    }
}

impl Into<PrimeExpr> for &Let {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Let(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Eq {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Eq {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Eq { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        Eq {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} == {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Eq {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Eq(self)
    }
}

impl Into<PrimeExpr> for &Eq {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Eq(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Ne {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Ne {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Ne { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        Ne {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Ne {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} != {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Ne {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Ne(self)
    }
}

impl Into<PrimeExpr> for &Ne {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Ne(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Lt {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Lt {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Lt { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Lt {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Lt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} < {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Lt {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Lt(self)
    }
}

impl Into<PrimeExpr> for &Lt {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Lt(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Le {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Le {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_le(self);
    }
}

impl Le {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Le { e1, e2 }
    }

    pub fn make<T: Into<PrimeExpr>>(e1: T, e2: T) -> Self {
        Le {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Le {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} <= {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Le {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Le(self)
    }
}

impl Into<PrimeExpr> for &Le {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Le(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Rem {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Rem {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_mod(self);
    }
}

impl Rem {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Rem { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Rem {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Rem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} % {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Rem {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Rem(self)
    }
}

impl Into<PrimeExpr> for &Rem {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Rem(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Min {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Min {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_min(self);
    }
}

impl Min {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Min { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Min {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Min {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "min({}, {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Min {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Min(self)
    }
}

impl Into<PrimeExpr> for &Min {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Min(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Max {
    e1: Arc<PrimeExpr>,
    e2: Arc<PrimeExpr>,
}

impl Accepter for Max {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_max(self);
    }
}

impl Max {
    pub fn new(e1: Arc<PrimeExpr>, e2: Arc<PrimeExpr>) -> Self {
        Max { e1, e2 }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>>(e1: A, e2: B) -> Self {
        Max {
            e1: e1.into().into(),
            e2: e2.into().into(),
        }
    }

    pub fn e1(&self) -> &PrimeExpr {
        &self.e1
    }

    pub fn e2(&self) -> &PrimeExpr {
        &self.e2
    }

    pub fn e1_(&self) -> &Arc<PrimeExpr> {
        &self.e1
    }

    pub fn e2_(&self) -> &Arc<PrimeExpr> {
        &self.e2
    }
}

impl Display for Max {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "max({}, {})", self.e1, self.e2)
    }
}

impl Into<PrimeExpr> for Max {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Max(self)
    }
}

impl Into<PrimeExpr> for &Max {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Max(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Reduce {
    pub(crate) identity: Arc<Vec<PrimeExpr>>,
    pub(crate) iter_vars: Arc<Vec<IterVar>>,
    pub(crate) expr: Arc<Vec<PrimeExpr>>,
    pub(crate) op: &'static str,
}

impl Accepter for Reduce {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_reduce(self);
    }
}

impl Reduce {
    pub fn new(
        expr: Arc<Vec<PrimeExpr>>,
        identity: Arc<Vec<PrimeExpr>>,
        iter_vars: Arc<Vec<IterVar>>,
        op: &'static str
    ) -> Self {
        Reduce {
            expr,
            identity,
            iter_vars,
            op,
        }
    }

    pub fn make<A: Into<PrimeExpr>, B: Into<PrimeExpr>, C: Into<IterVar>>(
        expr: Vec<A>,
        identity: Vec<B>,
        iter_vars: Vec<C>,
        op: &'static str
    ) -> Self {
        Reduce {
            expr: expr
                .into_iter()
                .map(|e| e.into().into())
                .collect::<Vec<PrimeExpr>>()
                .into(),
            identity: identity
                .into_iter()
                .map(|e| e.into().into())
                .collect::<Vec<PrimeExpr>>()
                .into(),
            iter_vars: iter_vars
                .into_iter()
                .map(|e| e.into())
                .collect::<Vec<IterVar>>()
                .into(),
            op,
        }
    }
    pub fn op(&self) -> &'static str {
        self.op
    }
    pub fn identity(&self) -> &Vec<PrimeExpr> {
        &self.identity
    }
    pub fn iter_vars(&self) -> &Vec<IterVar> {
        &self.iter_vars
    }
    pub fn identity_(&self) -> &Arc<Vec<PrimeExpr>> {
        &self.identity
    }
    pub fn expr(&self) -> &Vec<PrimeExpr> {
        &self.expr
    }
    pub fn expr_(&self) -> &Arc<Vec<PrimeExpr>> {
        &self.expr
    }
}

impl Display for Reduce {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "reduce")
    }
}

impl Into<PrimeExpr> for Reduce {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Reduce(self)
    }
}

impl Into<PrimeExpr> for &Reduce {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Reduce(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Malloc {
    dtype: Dtype,
    size: Arc<PrimeExpr>,
}

impl Accepter for Malloc {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_malloc(self);
    }
}

impl Malloc {
    pub fn new(dtype: Dtype, size: Arc<PrimeExpr>) -> Self {
        Malloc { dtype, size }
    }

    pub fn make<T: Into<PrimeExpr>>(dtype: Dtype, size: T) -> Self {
        Malloc {
            dtype,
            size: size.into().into(),
        }
    }

    pub fn dtype(&self) -> Dtype {
        self.dtype
    }

    pub fn size(&self) -> &PrimeExpr {
        &self.size
    }

    pub fn size_(&self) -> &Arc<PrimeExpr> {
        &self.size
    }
}

impl Display for Malloc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "malloc<{}>({})", self.dtype, self.size)
    }
}

impl Into<PrimeExpr> for Malloc {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Malloc(self)
    }
}

impl Into<PrimeExpr> for &Malloc {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Malloc(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Layout {
    dim: Arc<PrimeExpr>,
    shape: Arc<PrimeExpr>,
    strides: Arc<PrimeExpr>,
}

impl Accepter for Layout {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_layout(self);
    }
}

impl Layout {
    pub fn new(dim: Arc<PrimeExpr>, shape: Arc<PrimeExpr>, strides: Arc<PrimeExpr>) -> Self {
        Layout { dim, shape, strides }
    }

    pub fn make<T: Into<PrimeExpr>, U: Into<PrimeExpr>, V: Into<PrimeExpr>>(
        dim: T,
        shape: U,
        strides: V
    ) -> Self {
        Layout {
            dim: dim.into().into(),
            shape: shape.into().into(),
            strides: strides.into().into(),
        }
    }

    pub fn dim(&self) -> &PrimeExpr {
        &self.dim
    }

    pub fn shape(&self) -> &PrimeExpr {
        &self.shape
    }

    pub fn strides(&self) -> &PrimeExpr {
        &self.strides
    }

    pub fn dim_(&self) -> &Arc<PrimeExpr> {
        &self.dim
    }

    pub fn shape_(&self) -> &Arc<PrimeExpr> {
        &self.shape
    }

    pub fn strides_(&self) -> &Arc<PrimeExpr> {
        &self.strides
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Layout({}, {}, {})", self.dim, self.shape, self.strides)
    }
}

impl Into<PrimeExpr> for Layout {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Layout(self)
    }
}

impl Into<PrimeExpr> for &Layout {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Layout(self.clone())
    }
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub struct Alloca {
    dtype: Dtype,
    size: Arc<PrimeExpr>,
}

impl Accepter for Alloca {
    fn accept<V: IRVisitor>(&self, visitor: &V) {
        visitor.visit_alloca(self);
    }
}

impl Alloca {
    pub fn new(dtype: Dtype, size: Arc<PrimeExpr>) -> Self {
        Alloca { dtype, size }
    }

    pub fn make<T: Into<PrimeExpr>>(dtype: Dtype, size: T) -> Self {
        Alloca {
            dtype,
            size: size.into().into(),
        }
    }

    pub fn dtype(&self) -> Dtype {
        self.dtype
    }

    pub fn size(&self) -> &PrimeExpr {
        &self.size
    }

    pub fn size_(&self) -> &Arc<PrimeExpr> {
        &self.size
    }
}

impl Display for Alloca {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "alloca<{}>({})", self.dtype, self.size)
    }
}

impl Into<PrimeExpr> for Alloca {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Alloca(self)
    }
}

impl Into<PrimeExpr> for &Alloca {
    fn into(self) -> PrimeExpr {
        PrimeExpr::Alloca(self.clone())
    }
}
