use std::{ fmt::Display, sync::Arc };
use tensor_common::shape_utils::predict_reduce_shape;
use tensor_common::{
    layout::Layout,
    shape::Shape,
    shape_utils::{ predict_broadcast_shape, predict_broadcast_strides },
    strides::Strides,
};
use tensor_types::dtype::Dtype;

use crate::halide::exprs::Load;
use crate::registry::MANAGER;
use crate::{
    halide::{ exprs::Int, prime_expr::PrimeExpr, variable::Variable },
    op::OpType,
    registry::Closures,
};

use super::{
    _value::_Value,
    expr::Expr,
    func_type::Type,
    traits::{
        HlirAccepterMut,
        HlirAccepterMutate,
        HlirAcceptor,
        HlirMutVisitor,
        HlirMutateVisitor,
        HlirVisitor,
        IntoVar,
    },
};

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Value {
    dtype: Dtype,
    value: _Value,
}

impl Value {
    pub fn make<T: Into<_Value>>(dtype: Dtype, value: T) -> Self {
        Self {
            dtype,
            value: value.into(),
        }
    }
}

impl HlirAcceptor for Value {
    fn accept<V: HlirVisitor>(&self, visitor: &V) {
        visitor.visit_value(self);
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Tuple {
    values: Arc<Vec<Expr>>,
}

impl Tuple {
    pub fn make<T: IntoIterator<Item: Into<Expr>>>(values: T) -> Self {
        Self {
            values: values
                .into_iter()
                .map(|x| x.into().into())
                .collect::<Vec<Expr>>()
                .into(),
        }
    }
    pub fn values(&self) -> &[Expr] {
        &self.values
    }
}

impl HlirAcceptor for Tuple {
    fn accept<V: HlirVisitor>(&self, visitor: &V) {
        visitor.visit_tuple(self);
    }
}

impl Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "({})",
            self.values
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Str {
    value: Arc<String>,
}

impl Str {
    pub fn make(value: &str) -> Self {
        Self {
            value: Arc::new(value.into()),
        }
    }
    pub fn value(&self) -> &str {
        &self.value
    }
}

impl HlirAcceptor for Str {
    fn accept<V: HlirVisitor>(&self, visitor: &V) {
        visitor.visit_str(self);
    }
}

impl Display for Str {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl HlirAcceptor for Variable {
    fn accept<V: HlirVisitor>(&self, visitor: &V) {
        visitor.visit_variable(self);
    }
}

impl HlirAccepterMut for Variable {
    fn accept_mut<V: HlirMutVisitor>(&self, visitor: &mut V) {
        visitor.visit_variable(self);
    }
}

impl HlirAccepterMutate for Variable {
    fn accept_mutate<V: HlirMutateVisitor>(&self, visitor: &mut V) {
        visitor.visit_variable(self);
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Cast {
    expr: Arc<Expr>,
    dtype: Dtype,
}

impl Cast {
    pub fn make<T: Into<Expr>>(expr: T, dtype: Dtype) -> Self {
        Self { expr: expr.into().into(), dtype }
    }
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
    pub fn expr_(&self) -> &Arc<Expr> {
        &self.expr
    }
    pub fn dtype(&self) -> Dtype {
        self.dtype
    }
}

impl HlirAcceptor for Cast {
    fn accept<V: HlirVisitor>(&self, visitor: &V) {
        visitor.visit_cast(self);
    }
}

impl Display for Cast {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} as {})", self.expr, self.dtype)
    }
}

macro_rules! impl_binop {
    ($struct:ident, $visit_method:ident) => {
        impl $struct {
            pub fn make<T: Into<Expr>, U: Into<Expr>>(lhs: T, rhs: U) -> Self {
                Self { lhs: lhs.into().into(), rhs: rhs.into().into() }
            }
            pub fn lhs(&self) -> &Expr {
                &self.lhs
            }
            pub fn rhs(&self) -> &Expr {
                &self.rhs
            }
            pub fn lhs_(&self) -> &Arc<Expr> {
                &self.lhs
            }
            pub fn rhs_(&self) -> &Arc<Expr> {
                &self.rhs
            }
        }

        impl HlirAcceptor for $struct {
            fn accept<V: HlirVisitor>(&self, visitor: &V) {
                visitor.$visit_method(self);
            }
        }
    };
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct OpNode {
    name: Variable,
    args: Option<
        Arc<Vec<(String /* arg name */, String /* arg type */, String /* arg description */)>>
    >,
    num_inputs: i64,
    fn_type: Arc<Type>,
    registry_idx: usize,
    op_type: OpType,
}

impl OpNode {
    pub fn new<T: Into<Variable>>(name: T) -> Self {
        Self {
            name: name.into(),
            args: None,
            registry_idx: 0,
            num_inputs: 0,
            fn_type: Arc::new(Type::None),
            op_type: OpType::Opaque,
        }
    }
    pub fn var(&self) -> &Variable {
        &self.name
    }
    pub fn name(&self) -> &str {
        &self.name.name
    }
    pub fn registry_idx(&self) -> usize {
        self.registry_idx
    }
    pub fn set_registry_idx(&mut self, idx: usize) {
        self.registry_idx = idx;
    }

    pub fn set_num_inputs(&mut self, num_inputs: i64) {
        self.num_inputs = num_inputs;
    }
    pub fn add_argument<A: Into<String>, B: Into<String>, C: Into<String>>(
        &mut self,
        arg_name: A,
        arg_type: B,
        arg_desc: C
    ) {
        if let Some(args) = self.args.as_mut() {
            Arc::make_mut(args).push((arg_name.into(), arg_type.into(), arg_desc.into()));
        } else {
            self.args = Some(vec![(arg_name.into(), arg_type.into(), arg_desc.into())].into());
        }
    }

    pub fn set_op_type(&mut self, op_type: OpType) {
        self.op_type = op_type;
    }
}

impl Display for OpNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "OpNode({})", self.name)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Add {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Add {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} + {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Sub {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Sub {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} - {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Mul {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Mul {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} * {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Div {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} / {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Mod {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Mod {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} % {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Min {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Min {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "min({}, {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Max {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Max {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "max({}, {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Eq {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Eq {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} == {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Ne {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Ne {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} != {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Lt {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Lt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} < {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Le {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Le {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} <= {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Gt {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Gt {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} > {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Ge {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Ge {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} >= {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct And {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for And {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} && {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Or {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Or {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} || {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Xor {
    lhs: Arc<Expr>,
    rhs: Arc<Expr>,
}

impl Display for Xor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} ^ {})", self.lhs, self.rhs)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Not {
    expr: Arc<Expr>,
}

impl Not {
    pub fn make<T: Into<Expr>>(expr: T) -> Self {
        Self { expr: expr.into().into() }
    }
    pub fn expr(&self) -> &Expr {
        &self.expr
    }
    pub fn expr_(&self) -> &Arc<Expr> {
        &self.expr
    }
}

impl Display for Not {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "!{}", self.expr)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Call {
    op: Arc<Expr>,
    args: Arc<Vec<Expr>>,
}

impl Call {
    pub fn make<T: Into<Expr>, U: IntoIterator<Item: Into<Expr>>>(name: T, args: U) -> Self {
        Self {
            op: Arc::new(name.into()),
            args: args
                .into_iter()
                .map(|x| x.into().into())
                .collect::<Vec<Expr>>()
                .into(),
        }
    }
    pub fn name(&self) -> &Expr {
        &self.op
    }
    pub fn args(&self) -> &[Expr] {
        &self.args
    }
}

impl Display for Call {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.op,
            self.args
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Select {
    cond: Arc<Expr>,
    true_value: Arc<Expr>,
    false_value: Arc<Expr>,
}

impl Select {
    pub fn make<T: Into<Expr>, U: Into<Expr>, V: Into<Expr>>(
        cond: T,
        true_value: U,
        false_value: V
    ) -> Self {
        Self {
            cond: cond.into().into(),
            true_value: true_value.into().into(),
            false_value: false_value.into().into(),
        }
    }
    pub fn cond(&self) -> &Expr {
        &self.cond
    }
    pub fn true_value(&self) -> &Expr {
        &self.true_value
    }
    pub fn false_value(&self) -> &Expr {
        &self.false_value
    }
    pub fn cond_(&self) -> &Arc<Expr> {
        &self.cond
    }
    pub fn true_value_(&self) -> &Arc<Expr> {
        &self.true_value
    }
    pub fn false_value_(&self) -> &Arc<Expr> {
        &self.false_value
    }
}

impl Display for Select {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({} ? {} : {})", self.cond, self.true_value, self.false_value)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Let {
    var: Arc<Variable>,
    value: Arc<Expr>,
    body: Arc<Expr>,
}

impl Let {
    pub fn make<U: Into<Expr>, T: IntoVar, C: Into<Expr>>(var: T, value: U, body: C) -> Self {
        Self {
            var: var.into_var().into(),
            value: value.into().into(),
            body: body.into().into(),
        }
    }
    pub fn make_from_expr<T: Into<Variable>, U: Into<Expr>, C: Into<Expr>>(
        var: T,
        value: U,
        body: C
    ) -> Self {
        Self {
            var: var.into().into(),
            value: value.into().into(),
            body: body.into().into(),
        }
    }
    pub fn var(&self) -> &Variable {
        &self.var
    }
    pub fn value(&self) -> &Expr {
        &self.value
    }
    pub fn var_(&self) -> &Arc<Variable> {
        &self.var
    }
    pub fn value_(&self) -> &Arc<Expr> {
        &self.value
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn body_(&self) -> &Arc<Expr> {
        &self.body
    }
}

impl Display for Let {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.body.as_ref() {
            Expr::Let(_) | Expr::For(_) | Expr::While(_) | Expr::If(_) =>
                write!(f, "let {} = {};\n{}", self.var, self.value, self.body),
            Expr::None => write!(f, "let {} = {}", self.var, self.value),
            _ => write!(f, "let {} = {} in {}", self.var, self.value, self.body),
        }
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub enum Tensor {
    Reduce(ReduceTensor),
    Fuse(FuseTensor),
    Base(BaseTensor),
}

impl Tensor {
    pub fn shape(&self) -> &Shape {
        match self {
            Tensor::Reduce(reduce) => &reduce.inputs[0].shape(),
            Tensor::Fuse(fuse) => &fuse.shape,
            Tensor::Base(base) => &base.layout.shape(),
        }
    }
    pub fn dtype(&self) -> Dtype {
        match self {
            Tensor::Reduce(reduce) => reduce.dtype,
            Tensor::Fuse(fuse) => fuse.dtype,
            Tensor::Base(base) => base.dtype,
        }
    }

    pub fn id(&self) -> usize {
        match self {
            Tensor::Reduce(reduce) => reduce.id,
            Tensor::Fuse(fuse) => fuse.id,
            Tensor::Base(base) => base.id,
        }
    }

    pub fn make(shape: Shape, dtype: Dtype, id: usize) -> Self {
        Tensor::Base(BaseTensor {
            reduced_strides: vec![].into(),
            layout: Layout::from(shape),
            load_fn: |var, expr| { crate::halide::exprs::Call::make(var.name(), &[expr]) },
            id,
            dtype,
        })
    }

    pub fn make_unop<A: Into<Self>>(fn_name: &str, lhs: A, id: usize) -> Self {
        let func = MANAGER.lock().unwrap().get(fn_name).expect("Cannot find op").clone();
        let lhs: Self = lhs.into();
        match lhs {
            Tensor::Fuse(ref fuse) => {
                let mut new_fuse = fuse.clone();
                new_fuse.inputs = vec![lhs].into();
                new_fuse.func = func;
                Tensor::Fuse(new_fuse)
            }
            Tensor::Base(base) => {
                let new_shape = base.layout.shape().clone();
                let dtype = base.dtype;
                Tensor::Fuse(FuseTensor {
                    inputs: vec![Tensor::Base(base)].into(),
                    shape: new_shape,
                    func,
                    op_name: fn_name.to_string(),
                    dtype,
                    id,
                })
            }
            _ => panic!("Cannot make unop from non-fuse node"),
        }
    }

    pub fn make_binop<A: Into<Self>, B: Into<Self>>(
        fn_name: &str,
        lhs: A,
        rhs: B,
        id: usize
    ) -> Self {
        let func = MANAGER.lock().unwrap().get(fn_name).expect("Cannot find op").clone();
        let mut lhs: Self = lhs.into();
        let dtype = lhs.dtype();
        let mut rhs: Self = rhs.into();
        match (&mut lhs, &mut rhs) {
            (Tensor::Fuse(_lhs), Tensor::Fuse(_rhs)) => {
                let lhs_shape = &_lhs.shape;
                let rhs_shape = &_rhs.shape;
                let new_shape = predict_broadcast_shape(lhs_shape, rhs_shape).expect(
                    "Cannot broadcast shapes"
                );
                lhs.update_strides(&new_shape);
                rhs.update_strides(&new_shape);
                Tensor::Fuse(FuseTensor {
                    inputs: vec![lhs, rhs].into(),
                    shape: new_shape,
                    func,
                    op_name: fn_name.to_string(),
                    dtype,
                    id,
                })
            }
            (Tensor::Fuse(_lhs), Tensor::Base(_rhs)) => {
                let new_shape = predict_broadcast_shape(&_lhs.shape, &_rhs.layout.shape()).expect(
                    "Cannot broadcast shapes"
                );
                rhs.update_strides(&new_shape);
                lhs.update_strides(&new_shape);
                Tensor::Fuse(FuseTensor {
                    inputs: vec![lhs, rhs].into(),
                    shape: new_shape,
                    func,
                    op_name: fn_name.to_string(),
                    dtype,
                    id,
                })
            }
            (Tensor::Base(_lhs), Tensor::Fuse(_rhs)) => {
                let new_shape = predict_broadcast_shape(_lhs.layout.shape(), &_rhs.shape).expect(
                    "Cannot broadcast shapes"
                );
                lhs.update_strides(&new_shape);
                rhs.update_strides(&new_shape);
                Tensor::Fuse(FuseTensor {
                    inputs: vec![lhs, rhs].into(),
                    shape: new_shape,
                    func,
                    op_name: fn_name.to_string(),
                    dtype,
                    id,
                })
            }
            (Tensor::Base(_lhs), Tensor::Base(_rhs)) => {
                let new_shape = predict_broadcast_shape(
                    &_lhs.layout.shape(),
                    &_rhs.layout.shape()
                ).expect("Cannot broadcast shapes");
                lhs.update_strides(&new_shape);
                rhs.update_strides(&new_shape);
                Tensor::Fuse(FuseTensor {
                    inputs: vec![lhs, rhs].into(),
                    shape: new_shape,
                    func,
                    op_name: fn_name.to_string(),
                    dtype,
                    id,
                })
            }
            _ => panic!("Cannot make binop from non-fuse node"),
        }
    }

    pub fn update_strides(&mut self, new_shape: &Shape) {
        match self {
            Tensor::Fuse(fuse) => {
                for i in Arc::make_mut(&mut fuse.inputs).iter_mut() {
                    i.update_strides(new_shape);
                }
            }
            Tensor::Base(base) => {
                let new_strides = predict_broadcast_strides(new_shape, &base.layout);
                base.layout = Layout::new(new_shape.clone(), new_strides);
            }
            Tensor::Reduce(reduce) => {
                for i in Arc::make_mut(&mut reduce.inputs).iter_mut() {
                    i.update_strides(new_shape);
                }
            }
        }
    }

    pub fn make_reduce<A: Into<Self>, B: Into<Vec<Int>>>(
        fn_name: &str,
        lhs: A,
        axes: B,
        init: PrimeExpr,
        id: usize
    ) -> Self {
        let func = MANAGER.lock().unwrap().get(fn_name).expect("Cannot find op").clone();
        let mut lhs: Self = lhs.into();
        let axes: Vec<Int> = axes.into();
        let axes = axes
            .iter()
            .map(|x| x.value() as usize)
            .collect::<Vec<_>>();

        match &mut lhs {
            Tensor::Fuse(fuse) => {
                let res_shape = predict_reduce_shape(&fuse.shape, &axes).1;
                // the shape of fuse should all be the same
                let mut reduce_vars = vec![];
                for i in axes.iter() {
                    reduce_vars.push(Variable::new(format!("r{}_{}", fuse.id, i)));
                }
                for i in Arc::make_mut(&mut fuse.inputs).iter_mut() {
                    i.update_reduce_strides(&axes);
                }
                let shape: Shape = res_shape.into();
                let reduce = ReduceTensor {
                    inputs: fuse.inputs.clone(),
                    reduce_vars: reduce_vars.into(),
                    identity: init,
                    func: fuse.func.clone(),
                    op_name: fn_name.to_string(),
                    dtype: fuse.dtype,
                    id,
                    shape: shape.clone(),
                };
                let wrapper = FuseTensor {
                    shape: shape.clone(),
                    inputs: vec![Tensor::Reduce(reduce)].into(),
                    func,
                    op_name: fn_name.to_string(),
                    dtype: fuse.dtype,
                    id,
                };
                wrapper.into()
            }
            _ => panic!("Cannot make reduce from non-fuse node"),
        }
    }

    pub fn update_reduce_strides(&mut self, axes: &[usize]) {
        match self {
            Tensor::Reduce(reduce) =>
                for i in Arc::make_mut(&mut reduce.inputs).iter_mut() {
                    i.update_reduce_strides(axes);
                }
            Tensor::Fuse(fuse) => {
                for i in Arc::make_mut(&mut fuse.inputs).iter_mut() {
                    i.update_reduce_strides(axes);
                }
            }
            Tensor::Base(base) => {
                let (a_shape_cpy, _) = predict_reduce_shape(base.layout.shape(), &axes);
                let mut j = base.layout.ndim() - axes.len();
                let mut k = 0;
                let mut track_idx = 0;
                let mut transposed_axis = vec![0; base.layout.ndim()];
                for i in 0..base.layout.ndim() {
                    if a_shape_cpy[i] != 0 {
                        transposed_axis[k] = i;
                        k += 1;
                    } else {
                        transposed_axis[j] = axes[track_idx];
                        j += 1;
                        track_idx += 1;
                    }
                }
                transposed_axis[base.layout.ndim() - axes.len()..].sort();
                let mut transposed_shape = base.layout.shape().to_vec();
                for i in transposed_axis.iter() {
                    transposed_shape[*i] = base.layout.shape()[transposed_axis[*i]];
                }
                let mut transposed_strides = base.layout.strides().to_vec();
                for i in transposed_axis.iter() {
                    transposed_strides[*i] = base.layout.strides()[transposed_axis[*i]];
                }
                base.layout = Layout::new(
                    &transposed_shape[..base.layout.ndim() - axes.len()],
                    &transposed_strides[..base.layout.ndim() - axes.len()]
                );
                if base.reduced_strides.len() > 0 {
                    let mut new_reduced_strides = vec![];
                    for i in transposed_strides[transposed_strides.len() - axes.len()..].iter() {
                        new_reduced_strides.push(*i);
                    }
                    new_reduced_strides.extend(base.reduced_strides.iter());
                    base.reduced_strides = new_reduced_strides.into();
                } else {
                    base.reduced_strides = transposed_strides[
                        transposed_strides.len() - axes.len()..
                    ]
                        .to_vec()
                        .into();
                }
            }
        }
    }

    pub fn reshape(&mut self, res_shape: &Shape) {
        match self {
            Tensor::Reduce(reduce) => {
                for i in Arc::make_mut(&mut reduce.inputs).iter_mut() {
                    i.reshape(res_shape);
                }
            }
            Tensor::Fuse(fuse) => {
                for i in Arc::make_mut(&mut fuse.inputs).iter_mut() {
                    i.reshape(res_shape);
                }
                fuse.shape = res_shape.clone();
            }
            Tensor::Base(base) => {
                assert!(res_shape.size() == base.layout.size());
                if let Some(new_strides) = base.layout.is_reshape_possible(res_shape) {
                    base.layout = Layout::new(res_shape.clone(), new_strides);
                } else {
                    panic!("Cannot reshape base node");
                }
            }
        }
    }

    pub fn lower(
        &self,
        push_vars: bool,
        vars: &mut Vec<Variable>,
        map: &mut Vec<(PrimeExpr, PrimeExpr)>
    ) -> PrimeExpr {
        match self {
            Tensor::Base(base) => {
                let mut strides = base.layout.strides().to_vec();
                let reduced_strides = base.reduced_strides.to_vec();
                strides.extend(reduced_strides.iter());
                assert!(vars.len() == strides.len());
                let variable_name = Variable::from(format!("%{}", base.id));
                let indices = vars
                    .iter()
                    .zip(strides.iter())
                    .map(|(v, s)| v * Int::make(Dtype::I64, *s))
                    .reduce(|a, b| a + b);
                let load = Load::make(variable_name, indices.unwrap());
                load.into()
            }
            Tensor::Reduce(reduce) => {
                let mut exprs = vec![];
                for var in reduce.reduce_vars.iter() {
                    vars.push(var.clone());
                }
                for input in reduce.inputs.iter() {
                    exprs.push(input.lower(false, vars, map));
                }
                for _ in reduce.reduce_vars.iter() {
                    vars.pop();
                }
                let call = reduce.func.call_common(exprs);
                call.into()
            }
            Tensor::Fuse(fuse) => {
                if push_vars {
                    for i in 0..fuse.shape.len() {
                        vars.push(Variable::new(format!("i{}", i)));
                    }
                }
                let mut exprs = vec![];
                for input in fuse.inputs.iter() {
                    exprs.push(input.lower(false, vars, map));
                }
                if fuse.inputs.len() == 1 && fuse.inputs[0].is_reduce() {
                    exprs.push(fuse.inputs[0].to_reduce().identity);
                    let call = fuse.func.call_common(exprs);
                    let var = Variable::from(format!("%r{}", fuse.id));
                    map.push((var.clone().into(), call.into()));
                    return var.into();
                } else {
                    let call = fuse.func.call_common(exprs);
                    call.into()
                }
            }
        }
    }

    pub fn is_reduce(&self) -> bool {
        match self {
            Tensor::Reduce(_) => true,
            _ => false,
        }
    }

    pub fn to_reduce(&self) -> ReduceTensor {
        match self {
            Tensor::Reduce(reduce) => reduce.clone(),
            _ => panic!("Cannot convert non-reduce node to reduce node"),
        }
    }
}

impl Into<Tensor> for ReduceTensor {
    fn into(self) -> Tensor {
        Tensor::Reduce(self)
    }
}

impl Into<Tensor> for FuseTensor {
    fn into(self) -> Tensor {
        Tensor::Fuse(self)
    }
}

impl Display for Tensor {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Tensor(shape=({}), {})",
            self
                .shape()
                .iter()
                .map(|x| x.to_string())
                .collect::<Vec<String>>()
                .join(", "),
            self.dtype()
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct BaseTensor {
    layout: Layout,
    reduced_strides: Strides,
    load_fn: fn(Variable, PrimeExpr) -> crate::halide::exprs::Call,
    id: usize,
    dtype: Dtype,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct ReduceTensor {
    inputs: Arc<Vec<Tensor>>,
    reduce_vars: Arc<Vec<Variable>>,
    shape: Shape,
    identity: PrimeExpr,
    func: Closures,
    op_name: String,
    id: usize,
    dtype: Dtype,
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct FuseTensor {
    shape: Shape,
    inputs: Arc<Vec<Tensor>>,
    func: Closures,
    op_name: String,
    id: usize,
    dtype: Dtype,
}

impl Into<Tensor> for &Tensor {
    fn into(self) -> Tensor {
        self.clone()
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct ComputeNode {
    compute_expr: PrimeExpr,
    iter_vars: Arc<Vec<Variable>>,
    reduce_vars: Arc<Vec<Variable>>,
    strides: Option<Arc<Vec<Int>>>,
    shape: Shape,
    id: usize,
}

impl ComputeNode {
    pub fn compute_expr(&self) -> &PrimeExpr {
        &self.compute_expr
    }
}

impl Display for ComputeNode {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.compute_expr)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct TensorType {
    dtype: Dtype,
    layout: Arc<Layout>,
}

impl TensorType {
    pub fn make(dtype: Dtype, layout: Layout) -> Self {
        Self {
            dtype,
            layout: Arc::new(layout),
        }
    }
    pub fn dtype(&self) -> Dtype {
        self.dtype
    }
    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

impl Display for TensorType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Tensor(dtype={}, shape={:?}, strides={:?})",
            self.dtype,
            self.layout.shape().inner(),
            self.layout.strides().inner()
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Alloc {
    shape: Arc<Expr>,
    dtype: Dtype,
}

impl Alloc {
    pub fn make<T: Into<Expr>>(shape: T, dtype: Dtype) -> Self {
        Self {
            shape: shape.into().into(),
            dtype,
        }
    }
    pub fn shape(&self) -> &Expr {
        &self.shape
    }
    pub fn dtype(&self) -> Dtype {
        self.dtype
    }
}

impl Display for Alloc {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "alloc({}, {})", self.shape, self.dtype)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct If {
    cond: Arc<Expr>,
    then: Arc<Expr>,
    else_: Arc<Expr>,
}

impl If {
    pub fn make<T: Into<Expr>, U: Into<Expr>, V: Into<Expr>>(cond: T, then: U, else_: V) -> Self {
        Self {
            cond: cond.into().into(),
            then: then.into().into(),
            else_: else_.into().into(),
        }
    }
    pub fn cond(&self) -> &Expr {
        &self.cond
    }
    pub fn then(&self) -> &Expr {
        &self.then
    }
    pub fn else_(&self) -> &Expr {
        &self.else_
    }
    pub fn cond_(&self) -> &Arc<Expr> {
        &self.cond
    }
    pub fn then_(&self) -> &Arc<Expr> {
        &self.then
    }
    pub fn else__(&self) -> &Arc<Expr> {
        &self.else_
    }
}

impl Display for If {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "if {} {{\n{}}} else {{\n{}}}", self.cond, self.then, self.else_)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct For {
    var: Arc<Variable>,
    start: Arc<Expr>,
    end: Arc<Expr>,
    step: Arc<Expr>,
    body: Arc<Expr>,
}

impl For {
    pub fn make<T: IntoVar, U: Into<Expr>, V: Into<Expr>, W: Into<Expr>, X: Into<Expr>>(
        var: T,
        start: U,
        end: V,
        step: W,
        body: X
    ) -> Self {
        Self {
            var: var.into_var().into(),
            start: start.into().into(),
            end: end.into().into(),
            step: step.into().into(),
            body: body.into().into(),
        }
    }
    pub fn var(&self) -> &Variable {
        &self.var
    }
    pub fn start(&self) -> &Expr {
        &self.start
    }
    pub fn end(&self) -> &Expr {
        &self.end
    }
    pub fn step(&self) -> &Expr {
        &self.step
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl Display for For {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "for {} in {}..{} {{\n{}\n}}", self.var, self.start, self.end, self.body)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct While {
    cond: Arc<Expr>,
    body: Arc<Expr>,
}

impl While {
    pub fn make<T: Into<Expr>, U: Into<Expr>>(cond: T, body: U) -> Self {
        Self {
            cond: cond.into().into(),
            body: body.into().into(),
        }
    }
    pub fn cond(&self) -> &Expr {
        &self.cond
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
}

impl Display for While {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "while {} {{\n{}\n}}", self.cond, self.body)
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Function {
    name: Arc<String>,
    args: Arc<Vec<Variable>>,
    return_type: Arc<Expr>,
    body: Arc<Expr>,
}

impl Function {
    pub fn make<T: Into<String>, U: IntoIterator<Item: IntoVar>, W: Into<Expr>>(
        name: T,
        args: U,
        return_type: &Type,
        body: W
    ) -> Self {
        let ret_type: Expr = return_type.into();
        Self {
            name: Arc::new(name.into()),
            args: args
                .into_iter()
                .map(|x| x.into_var())
                .collect::<Vec<Variable>>()
                .into(),
            return_type: ret_type.into(),
            body: body.into().into(),
        }
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn args(&self) -> &[Variable] {
        &self.args
    }
    pub fn return_type(&self) -> &Expr {
        &self.return_type
    }
    pub fn body(&self) -> &Expr {
        &self.body
    }
    pub fn body_mut(&mut self) -> &mut Expr {
        Arc::make_mut(&mut self.body)
    }

    pub fn set_body<T: Into<Expr>>(&mut self, body: T) {
        self.body = body.into().into();
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "fn {}({}) -> {} {{\n{}\n}}",
            self.name,
            self.args
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", "),
            self.return_type,
            self.body
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Slice {
    var: Arc<Variable>,
    selections: Arc<Vec<(Expr, Expr, Expr)>>,
}

impl Slice {
    pub fn make<T: IntoVar, U, A, B, C>(var: T, selections: U) -> Self
        where A: Into<Expr>, B: Into<Expr>, C: Into<Expr>, U: IntoIterator<Item = (A, B, C)>
    {
        Self {
            var: var.into_var().into(),
            selections: Arc::new(
                selections
                    .into_iter()
                    .map(|x| (x.0.into(), x.1.into(), x.2.into()))
                    .collect()
            ),
        }
    }
    pub fn var(&self) -> &Variable {
        &self.var
    }
    pub fn selections(&self) -> &[(Expr, Expr, Expr)] {
        &self.selections
    }
}

impl Display for Slice {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}[{}]",
            self.var,
            self.selections
                .iter()
                .map(|(start, end, step)| format!("{}:{}:{}", start, end, step))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct Return {
    expr: Arc<Vec<Expr>>,
}

impl Return {
    pub fn make<T: IntoIterator<Item: Into<Expr>>>(expr: T) -> Self {
        Self {
            expr: expr
                .into_iter()
                .map(|x| x.into().into())
                .collect::<Vec<Expr>>()
                .into(),
        }
    }
    pub fn expr(&self) -> &[Expr] {
        &self.expr
    }

    pub fn expr_mut(&mut self) -> &mut Vec<Expr> {
        Arc::make_mut(&mut self.expr)
    }
}

impl Display for Return {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "return {}",
            self.expr
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[derive(Clone, PartialEq, Debug, Hash, Eq)]
pub struct CommonReduce {
    value: Arc<Expr>,
    axes: Arc<Vec<Expr>>,
    closure: Closures,
}

impl CommonReduce {
    pub fn make<T: Into<Expr>, U: IntoIterator<Item: Into<Expr>>>(
        value: T,
        axes: U,
        closure: Closures
    ) -> Self {
        Self {
            value: value.into().into(),
            axes: axes
                .into_iter()
                .map(|x| x.into().into())
                .collect::<Vec<Expr>>()
                .into(),
            closure,
        }
    }
    pub fn value(&self) -> &Expr {
        &self.value
    }
    pub fn axes(&self) -> &[Expr] {
        &self.axes
    }
    pub fn closure(&self) -> &Closures {
        &self.closure
    }
}

impl Display for CommonReduce {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}[{}]",
            self.value,
            self.axes
                .iter()
                .map(|x| format!("{}", x))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

macro_rules! impl_into_expr {
    ($struct:ident) => {
        impl Into<Expr> for $struct {
            fn into(self) -> Expr {
                Expr::$struct(self)
            }
        }
        
        impl Into<Expr> for &$struct {
            fn into(self) -> Expr {
                Expr::$struct(self.clone())
            }
        }

        impl Into<Expr> for &&$struct {
            fn into(self) -> Expr {
                Expr::$struct((*self).clone())
            }
        }
    };
}

impl_into_expr!(Value);
impl_into_expr!(Str);
impl_into_expr!(Variable);
impl_into_expr!(Cast);
impl_into_expr!(Add);
impl_into_expr!(Sub);
impl_into_expr!(Mul);
impl_into_expr!(Div);
impl_into_expr!(Mod);
impl_into_expr!(Min);
impl_into_expr!(Max);
impl_into_expr!(Eq);
impl_into_expr!(Ne);
impl_into_expr!(Lt);
impl_into_expr!(Le);
impl_into_expr!(Gt);
impl_into_expr!(Ge);
impl_into_expr!(And);
impl_into_expr!(Or);
impl_into_expr!(Xor);
impl_into_expr!(Not);
impl_into_expr!(Call);
impl_into_expr!(Select);
impl_into_expr!(Let);
impl_into_expr!(Alloc);
impl_into_expr!(If);
impl_into_expr!(For);
impl_into_expr!(While);
impl_into_expr!(Function);
impl_into_expr!(Tuple);
impl_into_expr!(TensorType);
impl_into_expr!(Slice);
impl_into_expr!(OpNode);
impl_into_expr!(Tensor);
impl_into_expr!(Return);

impl_binop!(Add, visit_add);
impl_binop!(Sub, visit_sub);
impl_binop!(Mul, visit_mul);
impl_binop!(Div, visit_div);
impl_binop!(Mod, visit_mod);
impl_binop!(Min, visit_min);
impl_binop!(Max, visit_max);
impl_binop!(Eq, visit_eq);
impl_binop!(Ne, visit_ne);
impl_binop!(Lt, visit_lt);
impl_binop!(Le, visit_le);
impl_binop!(Gt, visit_gt);
impl_binop!(Ge, visit_ge);
impl_binop!(And, visit_and);
impl_binop!(Or, visit_or);
impl_binop!(Xor, visit_xor);

impl IntoVar for Variable {
    fn into_var(self) -> Variable {
        self
    }
}

impl IntoVar for &Variable {
    fn into_var(self) -> Variable {
        self.clone()
    }
}

impl IntoVar for &[Variable] {
    fn into_var(self) -> Variable {
        Variable::make("")
    }
}

impl IntoVar for Arc<Variable> {
    fn into_var(self) -> Variable {
        self.as_ref().clone()
    }
}

impl IntoVar for &Arc<Variable> {
    fn into_var(self) -> Variable {
        self.as_ref().clone()
    }
}

impl IntoVar for &str {
    fn into_var(self) -> Variable {
        Variable::make(self)
    }
}

impl IntoVar for String {
    fn into_var(self) -> Variable {
        Variable::make(&self)
    }
}

impl IntoVar for &String {
    fn into_var(self) -> Variable {
        Variable::make(self)
    }
}
