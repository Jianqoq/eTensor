use std::{ collections::{ HashMap, HashSet }, panic::Location, sync::Arc };

use tensor_common::{
    shape::Shape,
    shape_utils::is_reshape_possible,
    strides_utils::shape_to_strides,
};
use tensor_types::{ dtype::Dtype, type_promote::NormalOut };

use crate::{
    halide::{
        exprs::{ Call, Int, Load },
        inplace_store_stmt::InplaceAdd,
        let_stmt::LetStmt,
        passes::const_fold::ConstFold,
        prime_expr::PrimeExpr,
        stmt::Stmt,
        store_stmt::StoreStmt,
        tensor_load::TensorLoad,
        variable::Variable,
    },
    iter_var::IterVar,
    te::{ hstrides::HStrides, idx_evaluator::IdxEvaluator },
    to_prim_expr::ToPrimeExpr,
};

use super::{
    rc_mut::RcMut,
    srg::Srg,
    srg_node::SrgNode,
    stages::{ Body, ReduceStage, Stage },
    strides_cal_helper::{
        binary_strides_cal,
        elementwise_strides_cal,
        reduce_strides_cal,
        slice_strides_cal,
    },
    tensor::{ StridesCal, Tensor },
};

#[derive(Clone)]
pub struct Context {
    pub(crate) nodes: RcMut<HashMap<usize, Tensor>>,
    pub(crate) vars: RcMut<HashSet<Variable>>,
    pub(crate) id: RcMut<usize>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            nodes: RcMut::new(HashMap::new()),
            id: RcMut::new(0),
            vars: RcMut::new(HashSet::new()),
        }
    }

    pub fn to_srg(self) -> Srg {
        let mut nodes = HashMap::new();
        for (id, node) in self.nodes.borrow().iter() {
            let srg_node = SrgNode {
                id: *id,
                shape: node.shape.clone(),
                inputs: node.inputs.clone(),
                outputs: Arc::new(
                    self.nodes
                        .borrow()
                        .iter()
                        .filter_map(|(k, v)| {
                            if v.inputs.contains(id) { Some(*k) } else { None }
                        })
                        .collect()
                ),
                strides_cal: Arc::new(|_| vec![]),
                span: node.span,
            };
            nodes.insert(*id, srg_node);
        }
        Srg {
            nodes,
            tensors: self.nodes.clone(),
        }
    }

    #[track_caller]
    pub fn var(&mut self, name: &str) -> Variable {
        let var = Variable::new(name.to_string());
        self.vars.borrow_mut().insert(var.clone());
        var
    }

    #[track_caller]
    pub fn placeholder(&mut self, shape: &[&dyn ToPrimeExpr], dtype: Dtype) -> Tensor {
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;
        let shape = Arc::new(
            shape
                .into_iter()
                .map(|x| x.to_prime_expr())
                .collect::<Vec<PrimeExpr>>()
        );
        let shape1 = shape.clone();
        let tensor = Tensor {
            shape: shape.clone(),
            dtype,
            span: Location::caller(),
            inputs: Arc::new(vec![]),
            id,
            strides_cal: Arc::new(move |_: Vec<StridesCal>| {
                let shape = shape1.clone();
                Arc::new(move |map: &HashMap<String, i64>| {
                    let real_shape = shape
                        .iter()
                        .map(|x| { IdxEvaluator::new(map).eval(x) })
                        .collect::<Vec<i64>>();
                    let hstrides = HStrides {
                        strides: shape_to_strides(&real_shape).inner().clone(),
                        reduced_dim: 0,
                        offset: 0,
                    };
                    vec![hstrides]
                })
            }),
            body_gen: Arc::new(move |_: Vec<Body>, _: bool, id: usize| {
                let shape = shape.clone();
                let body = Body::Stmt(
                    Stmt::LetStmt(
                        LetStmt::make(
                            &Variable::make(&format!("%{}_val", id)),
                            TensorLoad {
                                var: Variable::make(&format!("%{}", id)).into(),
                                axes: (0..shape.len())
                                    .map(|x| Variable::make(&format!("ax{}", x)).into())
                                    .collect::<Vec<PrimeExpr>>()
                                    .into(),
                                strides: (0..shape.len())
                                    .map(|idx|
                                        Load::make(
                                            Variable::make(&format!("%{}.s", id)),
                                            idx
                                        ).into()
                                    )
                                    .collect::<Vec<PrimeExpr>>()
                                    .into(),
                                hints: vec![].into(),
                            },
                            Stmt::None
                        )
                    ).into()
                );
                let stage = Stage {
                    dims: (0..shape.len())
                        .map(|x| IterVar::new(0i64, shape[x].clone(), 1i64, &format!("ax{}", x)))
                        .collect(),
                    bodys: vec![body],
                    id,
                };
                Body::Stage(stage)
            }),
        };
        self.nodes.borrow_mut().insert(id, tensor.clone());
        tensor
    }

    #[track_caller]
    pub fn reshape(&mut self, a: &Tensor, shape: &[&dyn ToPrimeExpr]) -> Tensor {
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;
        let new_shape = Arc::new(
            shape
                .into_iter()
                .map(|x| x.to_prime_expr())
                .collect::<Vec<PrimeExpr>>()
        );
        let prev_shape = a.shape.clone();
        let shape1 = new_shape.clone();
        let tensor = Tensor {
            shape: new_shape.clone().into(),
            dtype: a.dtype.clone(),
            span: Location::caller(),
            inputs: Arc::new(vec![a.id]),
            id,
            strides_cal: Arc::new(move |prev_fn: Vec<StridesCal>| {
                let prev_fn = prev_fn[0].clone();
                let new_shape = shape1.clone();
                let prev_shape = prev_shape.clone();
                Arc::new(move |map: &HashMap<String, i64>| {
                    let prev_strides = prev_fn.clone()(map);
                    let new_shape = new_shape
                        .iter()
                        .map(|x| { IdxEvaluator::new(map).eval(x) })
                        .collect::<Vec<i64>>();
                    let prev_shape = prev_shape
                        .iter()
                        .map(|x| { IdxEvaluator::new(map).eval(x) })
                        .collect::<Vec<i64>>();
                    let new_shape = Shape::from(new_shape);
                    let prev_shape = Shape::from(prev_shape);
                    let mut ret = vec![];
                    for i in prev_strides.into_iter() {
                        let masked_strides = &i[..i.strides.len() - i.reduced_dim];
                        assert_eq!(masked_strides.len(), prev_shape.len());
                        if
                            let Some(new_strides) = is_reshape_possible(
                                &prev_shape,
                                masked_strides,
                                &new_shape
                            )
                        {
                            let mut new = vec![];
                            for i in new_strides.inner().iter() {
                                new.push(*i);
                            }
                            for i in i[i.strides.len() - i.reduced_dim..].iter() {
                                new.push(*i);
                            }
                            assert!(new.len() == new_shape.len() + i.reduced_dim);
                            let new = HStrides {
                                strides: new,
                                reduced_dim: i.reduced_dim,
                                offset: i.offset,
                            };
                            ret.push(new);
                        } else {
                            panic!("Reshape not possible, {}", Location::caller());
                        }
                    }
                    ret
                })
            }),
            body_gen: Arc::new(move |inputs: Vec<Body>, is_output: bool, id: usize| {
                let shape = new_shape.clone();
                if is_output {
                    if let Body::Stage(stage) = &inputs[0] {
                        let mut stage = stage.clone();
                        let dims = (0..shape.len())
                            .map(|x|
                                IterVar::new(0i64, shape[x].clone(), 1i64, &format!("ax{}", x))
                            )
                            .collect::<Vec<IterVar>>();
                        stage.broadcast_new_dims(
                            &(0..shape.len())
                                .map(|x|
                                    Load::make(Variable::make(&format!("%{}.s", id)), x).into()
                                )
                                .collect::<Vec<PrimeExpr>>(),
                            &(0..shape.len())
                                .map(|x| Variable::make(&format!("ax{}", x)).into())
                                .collect::<Vec<PrimeExpr>>()
                        );
                        let body = Body::Stmt(
                            Stmt::StoreStmt(
                                StoreStmt::make(
                                    &Variable::make(&format!("%{}", id)),
                                    dims
                                        .iter()
                                        .enumerate()
                                        .map(
                                            |(idx, x)|
                                                x.var().to_prime_expr() *
                                                Load::make(&format!("%{}.s", id), idx).into()
                                        )
                                        .reduce(|acc, x| acc + x)
                                        .unwrap(),
                                    Variable::make(&format!("%{}_val", stage.id))
                                )
                            )
                        );
                        stage.bodys.push(body);
                        let stage = Stage {
                            dims,
                            bodys: stage.bodys.clone(),
                            id,
                        };
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                } else {
                    if let Body::Stage(stage) = &inputs[0] {
                        let mut stage = stage.clone();
                        let dims = (0..shape.len())
                            .map(|x|
                                IterVar::new(0i64, shape[x].clone(), 1i64, &format!("ax{}", x))
                            )
                            .collect::<Vec<IterVar>>();
                        stage.broadcast_new_dims(
                            &(0..shape.len())
                                .map(|x|
                                    Load::make(Variable::make(&format!("%{}.s", id)), x).into()
                                )
                                .collect::<Vec<PrimeExpr>>(),
                            &(0..shape.len())
                                .map(|x| Variable::make(&format!("ax{}", x)).into())
                                .collect::<Vec<PrimeExpr>>()
                        );
                        let body = Body::Stmt(
                            Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_val", id)),
                                    Variable::make(&format!("%{}_val", stage.id)),
                                    Stmt::None
                                )
                            ).into()
                        );
                        stage.bodys.push(body);
                        let stage = Stage {
                            dims,
                            bodys: stage.bodys.clone(),
                            id,
                        };
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                }
            }),
        };
        self.nodes.borrow_mut().insert(id, tensor.clone());
        tensor
    }

    #[track_caller]
    pub fn add(&mut self, a: &Tensor, b: &Tensor) -> Tensor {
        let lhs_shape = a.shape.clone();
        let rhs_shape = b.shape.clone();
        let mut res_shape = vec![];
        let (lhs_start, rhs_start) = if lhs_shape.len() < rhs_shape.len() {
            (0..rhs_shape.len() - lhs_shape.len()).for_each(|x| {
                res_shape.push(rhs_shape[x].clone());
            });
            (0, rhs_shape.len() - lhs_shape.len())
        } else if lhs_shape.len() > rhs_shape.len() {
            (0..lhs_shape.len() - rhs_shape.len()).for_each(|x| {
                res_shape.push(lhs_shape[x].clone());
            });
            (lhs_shape.len() - rhs_shape.len(), 0)
        } else {
            (0, 0)
        };
        let one = PrimeExpr::Int(Int::make(Dtype::I64, 1));
        lhs_shape[lhs_start..]
            .iter()
            .zip(rhs_shape[rhs_start..].iter())
            .for_each(|(x, y)| {
                if x == &one {
                    res_shape.push(y.clone());
                } else if y == &one {
                    res_shape.push(x.clone());
                } else if x == y {
                    res_shape.push(x.clone());
                } else {
                    panic!("Incompatible shapes. {} and {}", x, y);
                }
            });
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;
        let res_shape = Arc::new(res_shape);
        let res_shape1 = res_shape.clone();
        let ret = Tensor {
            shape: res_shape.clone(),
            inputs: Arc::new(vec![a.id, b.id]),
            span: Location::caller(),
            id,
            dtype: a.dtype._add(b.dtype),
            strides_cal: Arc::new(move |prev_fn: Vec<StridesCal>| {
                let lhs_shape = lhs_shape.clone();
                let rhs_shape = rhs_shape.clone();
                let res_shape = res_shape1.clone();
                binary_strides_cal(
                    lhs_shape,
                    rhs_shape,
                    res_shape,
                    prev_fn[0].clone(),
                    prev_fn[1].clone()
                )
            }),
            body_gen: Arc::new(move |inputs: Vec<Body>, is_output: bool, id: usize| {
                let lhs = &inputs[0];
                let rhs = &inputs[1];
                let dims = res_shape
                    .iter()
                    .enumerate()
                    .map(|(idx, x)| IterVar::new(0i64, x.clone(), 1i64, &format!("ax{}", idx)))
                    .collect::<Vec<IterVar>>();
                if is_output {
                    match (lhs, rhs) {
                        (Body::Stage(lhs), Body::Stage(rhs)) => {
                            let mut lhs_bodys = lhs.bodys.clone();
                            let rhs_bodys = rhs.bodys.clone();
                            lhs_bodys.extend(rhs_bodys);
                            let sotre_add = Stmt::StoreStmt(
                                StoreStmt::make(
                                    &Variable::make(&format!("%{}", id)),
                                    dims
                                        .iter()
                                        .enumerate()
                                        .map(
                                            |(idx, x)|
                                                x.var().to_prime_expr() *
                                                Load::make(&format!("%{}.s", id), idx).into()
                                        )
                                        .reduce(|acc, x| acc + x)
                                        .unwrap(),
                                    Variable::make(&format!("%{}_val", lhs.id)) +
                                        Variable::make(&format!("%{}_val", rhs.id))
                                )
                            );
                            lhs_bodys.push(Body::Stmt(sotre_add));
                            let stage = Stage {
                                dims,
                                bodys: lhs_bodys,
                                id,
                            };
                            Body::Stage(stage)
                        }
                        _ => panic!("input is not a stage"),
                    }
                } else {
                    match (lhs, rhs) {
                        (Body::Stage(lhs), Body::Stage(rhs)) => {
                            let mut lhs_bodys = lhs.bodys.clone();
                            let rhs_bodys = rhs.bodys.clone();
                            lhs_bodys.extend(rhs_bodys);
                            let add = Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_val", id)),
                                    Variable::make(&format!("%{}_val", lhs.id)) +
                                        Variable::make(&format!("%{}_val", rhs.id)),
                                    Stmt::None
                                )
                            );
                            lhs_bodys.push(Body::Stmt(add));
                            let stage = Stage {
                                dims,
                                bodys: lhs_bodys,
                                id,
                            };
                            Body::Stage(stage)
                        }
                        _ => panic!("input is not a stage"),
                    }
                }
            }),
        };
        self.nodes.borrow_mut().insert(id, ret.clone());
        ret
    }

    #[track_caller]
    pub fn sum(&mut self, a: &Tensor, init: &dyn ToPrimeExpr, axes: &[i64]) -> Tensor {
        let mut axes = axes.to_vec();
        for i in axes.iter_mut() {
            if *i < 0 {
                *i += a.shape.len() as i64;
            }
        }
        axes.sort();

        let mut res_shape = vec![];
        for i in 0..a.shape.len() as i64 {
            if !axes.contains(&i) {
                res_shape.push(a.shape[i as usize].clone());
            }
        }
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;

        let a_shape = a.shape.clone();
        let res_shape = Arc::new(res_shape);
        let init = init.to_prime_expr();
        let axes = Arc::new(axes);
        let axes1 = axes.clone();
        let ret = Tensor {
            shape: res_shape.clone(),
            inputs: Arc::new(vec![a.id]),
            span: Location::caller(),
            id,
            dtype: a.dtype.clone(),
            strides_cal: Arc::new(move |prev_fn: Vec<StridesCal>| {
                reduce_strides_cal(
                    prev_fn[0].clone(),
                    axes1
                        .iter()
                        .map(|x| *x as usize)
                        .collect()
                )
            }),
            body_gen: Arc::new(move |inputs: Vec<Body>, is_output: bool, id: usize| {
                let axes = axes
                    .iter()
                    .map(|x| *x as usize)
                    .collect::<Vec<usize>>();
                if is_output {
                    if let Body::Stage(stage) = &inputs[0] {
                        let mut stage = stage.clone();
                        let mut dims = (0..res_shape.len())
                            .map(|x| Variable::make(&format!("ax{}", x)).into())
                            .collect::<Vec<PrimeExpr>>();
                        let mut red_dims = vec![];
                        for i in axes.iter() {
                            red_dims.push(Variable::make(&format!("{}red{}", id, i)).into());
                        }
                        dims.extend(red_dims);
                        stage.broadcast_new_dims(
                            &(0..a_shape.len())
                                .map(|x|
                                    Load::make(Variable::make(&format!("%{}.s", id)), x).into()
                                )
                                .collect::<Vec<PrimeExpr>>(),
                            &dims
                        );
                        let mut bodys = stage.bodys.clone();
                        bodys.push(
                            Body::Stmt(
                                Stmt::InplaceAdd(
                                    InplaceAdd::make(
                                        &Variable::make(&format!("%{}_val", id)),
                                        Variable::make(&format!("%{}_val", stage.id))
                                    )
                                )
                            )
                        );
                        stage.dims = (0..a_shape.len())
                            .filter(|x| !axes.contains(x))
                            .enumerate()
                            .map(|(idx, x)|
                                IterVar::new(0i64, a_shape[x].clone(), 1i64, &format!("ax{}", idx))
                            )
                            .collect();
                        let all_parent_dims = &stage.dims;
                        let red_axes = axes
                            .iter()
                            .map(|x|
                                IterVar::new(
                                    0i64,
                                    a_shape[*x].clone(),
                                    1i64,
                                    &format!("{}red{}", id, x)
                                )
                            )
                            .collect::<Vec<IterVar>>();
                        let reduce_stage = ReduceStage {
                            dims: red_axes.clone(),
                            bodys,
                            id,
                            inits: vec![
                                Body::Stmt(
                                    Stmt::LetStmt(
                                        LetStmt::make(
                                            &Variable::make(&format!("%{}_val", id)),
                                            init.clone(),
                                            Stmt::None
                                        )
                                    ).into()
                                )
                            ],
                            posts: vec![
                                Body::Stmt(
                                    Stmt::StoreStmt(
                                        StoreStmt::make(
                                            &Variable::make(&format!("%{}", id)),
                                            all_parent_dims
                                                .iter()
                                                .enumerate()
                                                .map(
                                                    |(idx, x)|
                                                        PrimeExpr::Variable(x.var().clone()) *
                                                        Load::make(
                                                            &format!("%{}.s", id),
                                                            idx
                                                        ).into()
                                                )
                                                .reduce(|acc, x| acc + x)
                                                .unwrap_or((0i64).into()),
                                            Variable::make(&format!("%{}_val", id))
                                        )
                                    )
                                )
                            ],
                            input: stage.id,
                        };
                        stage.bodys = vec![Body::ReduceStage(reduce_stage)];
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                } else {
                    if let Body::Stage(stage) = &inputs[0] {
                        let mut stage = stage.clone();
                        let mut dims = (0..res_shape.len())
                            .map(|x| Variable::make(&format!("ax{}", x)).into())
                            .collect::<Vec<PrimeExpr>>();
                        let mut red_dims = vec![];
                        for i in axes.iter() {
                            red_dims.push(Variable::make(&format!("{}red{}", id, i)).into());
                        }
                        dims.extend(red_dims);
                        stage.broadcast_new_dims(
                            &(0..a_shape.len())
                                .map(|x|
                                    Load::make(Variable::make(&format!("%{}.s", id)), x).into()
                                )
                                .collect::<Vec<PrimeExpr>>(),
                            &dims
                        );
                        let mut bodys = stage.bodys.clone();
                        bodys.push(
                            Body::Stmt(
                                Stmt::InplaceAdd(
                                    InplaceAdd::make(
                                        &Variable::make(&format!("%{}_val", id)),
                                        Variable::make(&format!("%{}_val", stage.id))
                                    )
                                )
                            )
                        );
                        stage.dims = (0..a_shape.len())
                            .filter(|x| !axes.contains(x))
                            .enumerate()
                            .map(|(idx, x)|
                                IterVar::new(0i64, a_shape[x].clone(), 1i64, &format!("ax{}", idx))
                            )
                            .collect();
                        let red_axes = axes
                            .iter()
                            .map(|x|
                                IterVar::new(
                                    0i64,
                                    a_shape[*x].clone(),
                                    1i64,
                                    &format!("{}red{}", id, x)
                                )
                            )
                            .collect::<Vec<IterVar>>();
                        let reduce_stage = ReduceStage {
                            dims: red_axes.clone(),
                            bodys,
                            id,
                            inits: vec![
                                Body::Stmt(
                                    Stmt::LetStmt(
                                        LetStmt::make(
                                            &Variable::make(&format!("%{}_val", id)),
                                            init.clone(),
                                            Stmt::None
                                        )
                                    ).into()
                                )
                            ],
                            posts: vec![],
                            input: stage.id,
                        };
                        stage.bodys = vec![Body::ReduceStage(reduce_stage)];
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                }
            }),
        };
        self.nodes.borrow_mut().insert(id, ret.clone());
        ret
    }

    #[track_caller]
    pub fn sin(&mut self, a: &Tensor) -> Tensor {
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;
        let ret = Tensor {
            shape: a.shape.clone(),
            inputs: Arc::new(vec![a.id]),
            span: Location::caller(),
            id,
            dtype: a.dtype.clone(),
            strides_cal: Arc::new(move |prev_fn: Vec<StridesCal>| {
                elementwise_strides_cal(prev_fn[0].clone())
            }),
            body_gen: Arc::new(|inputs: Vec<Body>, is_output: bool, id: usize| {
                if is_output {
                    if let Body::Stage(stage) = &inputs[0] {
                        let mut stage = stage.clone();
                        let body = Body::Stmt(
                            Stmt::StoreStmt(
                                StoreStmt::make(
                                    &Variable::make(&format!("%{}", id)),
                                    stage.dims
                                        .iter()
                                        .enumerate()
                                        .map(
                                            |(idx, x)|
                                                x.var().to_prime_expr() *
                                                Load::make(&format!("%{}.s", id), idx).into()
                                        )
                                        .reduce(|acc, x| acc + x)
                                        .unwrap(),
                                    Variable::make(&format!("%{}_val", stage.id))
                                )
                            )
                        );
                        stage.bodys.push(body);
                        let stage = Stage {
                            dims: stage.dims.clone(),
                            bodys: stage.bodys.clone(),
                            id,
                        };
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                } else {
                    if let Body::Stage(stage) = &inputs[0] {
                        let body = Body::Stmt(
                            Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_val", id)),
                                    Call::make(
                                        "sin",
                                        &[
                                            Load::make(
                                                Variable::make(&format!("%{}_val", stage.id)),
                                                0
                                            ),
                                        ]
                                    ),
                                    Stmt::None
                                )
                            ).into()
                        );
                        let mut stage_bodys = stage.bodys.clone();
                        stage_bodys.push(body);
                        let stage = Stage {
                            dims: stage.dims.clone(),
                            bodys: stage_bodys,
                            id,
                        };
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                }
            }),
        };
        self.nodes.borrow_mut().insert(id, ret.clone());
        ret
    }

    #[track_caller]
    pub fn slice(
        &mut self,
        a: &Tensor,
        selections: &[
            (&dyn ToPrimeExpr /*begin */, &dyn ToPrimeExpr /*end */, &dyn ToPrimeExpr /*step */)
        ]
    ) -> Tensor {
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;
        let one = PrimeExpr::Int(Int::make(Dtype::I64, 1i64));
        let zero = PrimeExpr::Int(Int::make(Dtype::I64, 0i64));
        let selections = selections
            .iter()
            .map(|(begin, end, step)| {
                let mut const_fold = ConstFold::new();
                (
                    const_fold.const_fold(begin.to_prime_expr()),
                    const_fold.const_fold(end.to_prime_expr()),
                    const_fold.const_fold(step.to_prime_expr()),
                )
            })
            .collect::<Vec<(PrimeExpr, PrimeExpr, PrimeExpr)>>();
        let new_shape = Arc::new(
            selections
                .iter()
                .map(|(begin, end, step)| {
                    if begin == &zero && step == &one {
                        end.clone()
                    } else if step == &one {
                        end.clone() - begin.clone()
                    } else {
                        (end.clone() - begin.clone()) / step.clone()
                    }
                })
                .map(|x| {
                    let mut const_fold = ConstFold::new();
                    const_fold.const_fold(x)
                })
                .collect::<Vec<_>>()
        );
        let slice = Arc::new(selections.clone());
        let shape = new_shape.clone();
        let ret = Tensor {
            shape: new_shape.clone(),
            inputs: Arc::new(vec![a.id]),
            span: Location::caller(),
            dtype: a.dtype.clone(),
            id,
            strides_cal: Arc::new(move |prev_fn: Vec<StridesCal>| {
                let new_shape = new_shape.clone();
                let selections = selections.clone();
                slice_strides_cal(new_shape, selections, prev_fn[0].clone())
            }),
            body_gen: Arc::new(move |inputs: Vec<Body>, is_output: bool, id: usize| {
                if is_output {
                    if let Body::Stage(stage) = &inputs[0] {
                        let stage = stage.clone();
                        let dims = (0..stage.dims.len())
                            .zip(slice.iter())
                            .map(|(idx, (start, end, step))|
                                IterVar::new(
                                    0i64,
                                    (end -
                                        start +
                                        step -
                                        PrimeExpr::Int(Int::make(Dtype::I64, 1))) /
                                        step.clone(),
                                    1i64,
                                    &format!("ax{}", idx)
                                )
                            )
                            .collect::<Vec<IterVar>>();
                        let offset_body = Body::Stmt(
                            Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_ptr", id)),
                                    PrimeExpr::Variable(Variable::make(&format!("%{}", stage.id))) +
                                        PrimeExpr::Variable(
                                            Variable::make(&format!("%{}_offset", id))
                                        ),
                                    Stmt::None
                                )
                            )
                        );
                        let body = Body::Stmt(
                            Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_val", id)),
                                    TensorLoad {
                                        var: Variable::make(&format!("%{}_ptr", id)).into(),
                                        axes: (0..shape.len())
                                            .map(|x| Variable::make(&format!("ax{}", x)).into())
                                            .collect::<Vec<PrimeExpr>>()
                                            .into(),
                                        strides: (0..shape.len())
                                            .map(|idx|
                                                Load::make(
                                                    Variable::make(&format!("%{}.s", stage.id)),
                                                    idx
                                                ).into()
                                            )
                                            .collect::<Vec<PrimeExpr>>()
                                            .into(),
                                        hints: vec![].into(),
                                    },
                                    Stmt::None
                                )
                            ).into()
                        );
                        let store_body = Body::Stmt(
                            Stmt::StoreStmt(
                                StoreStmt::make(
                                    &Variable::make(&format!("%{}", id)),
                                    dims
                                        .iter()
                                        .enumerate()
                                        .map(
                                            |(idx, x)|
                                                x.var().to_prime_expr() *
                                                Load::make(&format!("%{}.s", id), idx).into()
                                        )
                                        .reduce(|acc, x| acc + x)
                                        .unwrap(),
                                    Variable::make(&format!("%{}_val", stage.id))
                                )
                            )
                        );
                        let stage = Stage {
                            dims,
                            bodys: vec![offset_body, body, store_body],
                            id,
                        };
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                } else {
                    if let Body::Stage(stage) = &inputs[0] {
                        let stage = stage.clone();
                        let dims = (0..stage.dims.len())
                            .zip(slice.iter())
                            .map(|(idx, (start, end, step))|
                                IterVar::new(
                                    0i64,
                                    (end -
                                        start +
                                        step -
                                        PrimeExpr::Int(Int::make(Dtype::I64, 1))) /
                                        step.clone(),
                                    1i64,
                                    &format!("ax{}", idx)
                                )
                            )
                            .collect::<Vec<IterVar>>();
                        let offset_body = Body::Stmt(
                            Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_ptr", id)),
                                    PrimeExpr::Variable(Variable::make(&format!("%{}", stage.id))) +
                                        PrimeExpr::Variable(
                                            Variable::make(&format!("%{}_offset", id))
                                        ),
                                    Stmt::None
                                )
                            )
                        );
                        let body = Body::Stmt(
                            Stmt::LetStmt(
                                LetStmt::make(
                                    &Variable::make(&format!("%{}_val", id)),
                                    TensorLoad {
                                        var: Variable::make(&format!("%{}_ptr", id)).into(),
                                        axes: (0..shape.len())
                                            .map(|x| Variable::make(&format!("ax{}", x)).into())
                                            .collect::<Vec<PrimeExpr>>()
                                            .into(),
                                        strides: (0..shape.len())
                                            .map(|idx|
                                                Load::make(
                                                    Variable::make(&format!("%{}.s", stage.id)),
                                                    idx
                                                ).into()
                                            )
                                            .collect::<Vec<PrimeExpr>>()
                                            .into(),
                                        hints: vec![].into(),
                                    },
                                    Stmt::None
                                )
                            ).into()
                        );
                        let stage = Stage {
                            dims,
                            bodys: vec![offset_body, body],
                            id,
                        };
                        Body::Stage(stage)
                    } else {
                        panic!("input is not a stage");
                    }
                }
            }),
        };
        self.nodes.borrow_mut().insert(id, ret.clone());
        ret
    }
}
