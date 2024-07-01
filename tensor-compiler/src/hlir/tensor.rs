#![allow(unused_imports)]
use std::{fmt::{Display, Formatter}, sync::Arc};

use hashbrown::HashMap;
use tensor_common::{ axis::{ process_axes, Axis }, shape::Shape };
use tensor_types::dtype::Dtype;

use crate::{
    halide::{
        assign_stmt::AssignStmt,
        exprs::{ Add, Gt, Int, Load, Lt, Max, Min, Mul },
        if_stmt::IfThenElse,
        inplace_store_stmt::InplaceAdd,
        let_stmt::LetStmt,
        loop_utils::build_nested::build_nested_for,
        prime_expr::PrimeExpr,
        seq_stmt::Seq,
        stmt::Stmt,
        store_stmt::StoreStmt,
        variable::Variable,
    }, hlir::input_visitor::InputVisitor, iter_val::IterVar
};

#[derive(Clone)]
pub struct Tensor {
    shape: Arc<Vec<IterVar>>,
    op: Arc<dyn Fn(Vec<PrimeExpr>) -> PrimeExpr>,
    name: Arc<String>,
    inputs: Arc<Vec<Arc<Tensor>>>,
}

impl std::fmt::Debug for Tensor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Tensor")
            .field("shape", &self.shape)
            .field("name", &self.name)
            .field("inputs", &self.inputs)
            .finish()
    }
}

impl Tensor {
    pub fn ndim(&self) -> usize {
        self.shape.len()
    }
    pub fn sum<AXIS: Into<Axis>>(&self, axes: AXIS, name: &str) -> Self {
        let axes: Vec<usize> = process_axes(axes, self.ndim()).unwrap();
        let _a = self.clone();
        let res_shape = self.shape
            .iter()
            .enumerate()
            .filter(|(i, _)| !axes.contains(i))
            .map(|(_, x)| x.clone())
            .collect::<Vec<_>>();

        // let c = compute([&n], name, move |[i]| {
        //     sum(
        //         [_a.slice([i, Variable::make("k").into()])],
        //         [Int::make(Dtype::BF16, 0)],
        //         [(0, &m, 1, "k")]
        //     )
        // });
        todo!()
    }
}

impl Display for Tensor {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Eq for Tensor {}

impl std::hash::Hash for Tensor {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.shape.hash(state);
        let ptr = Arc::as_ptr(&self.op);
        ptr.hash(state);
        self.name.hash(state);
    }
}

impl PartialEq for Tensor {
    fn eq(&self, other: &Self) -> bool {
        self.shape == other.shape && Arc::ptr_eq(&self.op, &other.op) && self.name == other.name
    }
}

impl Tensor {
    pub fn placeholder<T: IntoIterator<Item: Into<PrimeExpr>>>(shape: T, name: &str) -> Self {
        let tensor_name = name.to_string();
        let iter_vars = shape
            .into_iter()
            .map(|x| x.into())
            .enumerate()
            .map(|(i, x)| {
                IterVar::new(
                    Int::make(Dtype::I64, 0),
                    x,
                    Int::make(Dtype::I64, 1),
                    Variable::new(format!("ax{}", i))
                )
            })
            .collect::<Vec<_>>();
        Self {
            shape: Arc::new(iter_vars),
            op: Arc::new(move |vec| {
                Load::make(
                    Variable::new(tensor_name.to_string()),
                    vec
                        .iter()
                        .map(|x| x.clone())
                        .reduce(|acc, x| acc + x)
                        .unwrap()
                ).into()
            }),
            name: name.to_string().into(),
            inputs: vec![].into(),
        }
    }
    pub fn slice<T: IntoIterator<Item: Into<PrimeExpr>>>(&self, indices: T) -> PrimeExpr {
        Load::make(
            Variable::make(&self.name),
            indices
                .into_iter()
                .map(|x| x.into())
                .reduce(|acc, x| acc + x)
                .unwrap()
        ).into()
    }
}

pub fn compute<const N: usize, F, T: Into<PrimeExpr> + Clone>(
    res_shape: [T; N],
    name: &str,
    op: F
) -> Tensor
    where F: Fn([PrimeExpr; N]) -> PrimeExpr + 'static
{
    let new_fn = move |vec: Vec<PrimeExpr>| -> PrimeExpr { op(vec.try_into().unwrap()) };
    let iter_vars = res_shape
        .iter()
        .enumerate()
        .map(|(i, x)| {
            IterVar::new(
                Int::make(Dtype::I64, 0),
                x.clone(),
                Int::make(Dtype::I64, 1),
                Variable::new(format!("ax{}", i))
            )
        })
        .collect::<Vec<_>>();
    let val = new_fn(
        iter_vars
            .iter()
            .map(|x| x.var().clone().into())
            .collect::<Vec<_>>()
    );
    // let mut input_visitor = InputVisitor::visit(&val);
    
    todo!()
    // Tensor {
    //     shape: Arc::new(iter_vars),
    //     op: Arc::new(new_fn),
    //     name: name.to_string().into(),
    // }
}

pub struct Schedule {
    ops: HashMap<Tensor, Arc<dyn Fn(Vec<PrimeExpr>) -> PrimeExpr>>,
}

impl Schedule {
    pub fn create(tensors: Vec<Tensor>) -> Self {
        let mut ops = HashMap::new();
        for tensor in tensors {
            let op = tensor.op.clone();
            ops.insert(tensor, op);
        }
        Self { ops }
    }
    pub fn lower(&self) -> Vec<Stmt> {
        let mut stmts = Vec::new();
        for (tensor, op) in &self.ops {
            let shape = tensor.shape.clone();
            let name = tensor.name.clone();
            let expr = op(
                shape
                    .iter()
                    .map(|x| x.var().clone().into())
                    .collect()
            );
            let mut main_stmt: Vec<Stmt> = vec![];
            match expr {
                PrimeExpr::Reduce(reduce) => {
                    let indices = &shape
                        .iter()
                        .map(|x| x.var().into())
                        .reduce(|acc: PrimeExpr, x| acc + x)
                        .unwrap();
                    let iter_vars = reduce.iter_vars();
                    let fors = match reduce.op() {
                        "sum" => {
                            assert!(reduce.identity().len() == 1);
                            assert!(reduce.expr().len() == 1);
                            let out_name = Variable::make(&format!("output_{}", name));
                            main_stmt.push(
                                StoreStmt::make(
                                    &out_name,
                                    indices,
                                    &reduce.identity()[0].clone()
                                ).into()
                            );
                            build_nested_for(
                                iter_vars,
                                InplaceAdd::make(Load::make(&out_name, indices), &reduce.expr()[0])
                            )
                        }
                        "min" => {
                            assert!(reduce.identity().len() == 1);
                            assert!(reduce.expr().len() == 1);
                            main_stmt.push(
                                StoreStmt::make(
                                    &Variable::make(&format!("output_{}", name)),
                                    indices,
                                    &reduce.identity()[0].clone()
                                ).into()
                            );
                            build_nested_for(
                                iter_vars,
                                StoreStmt::make(
                                    &Variable::make(&format!("output_{}", name)),
                                    indices,
                                    &Min::make(
                                        &Load::make(
                                            Variable::make(&format!("output_{}", name)),
                                            indices
                                        ),
                                        &reduce.expr()[0]
                                    )
                                )
                            )
                        }
                        "max" => {
                            assert!(reduce.identity().len() == 1);
                            assert!(reduce.expr().len() == 1);
                            main_stmt.push(
                                StoreStmt::make(
                                    &Variable::make(&format!("output_{}", name)),
                                    indices,
                                    &reduce.identity()[0].clone()
                                ).into()
                            );
                            build_nested_for(
                                iter_vars,
                                StoreStmt::make(
                                    &Variable::make(&format!("output_{}", name)),
                                    indices,
                                    &Max::make(
                                        &Load::make(
                                            Variable::make(&format!("output_{}", name)),
                                            indices
                                        ),
                                        &reduce.expr()[0]
                                    )
                                )
                            )
                        }
                        "prod" => {
                            assert!(reduce.identity().len() == 1);
                            assert!(reduce.expr().len() == 1);
                            main_stmt.push(
                                StoreStmt::make(
                                    &Variable::make(&format!("output_{}", name)),
                                    indices,
                                    &reduce.identity()[0].clone()
                                ).into()
                            );
                            build_nested_for(
                                iter_vars,
                                StoreStmt::make(
                                    &Variable::make(&format!("output_{}", name)),
                                    indices,
                                    &Mul::make(
                                        &Load::make(
                                            Variable::make(&format!("output_{}", name)),
                                            indices
                                        ),
                                        &reduce.expr()[0]
                                    )
                                )
                            )
                        }
                        "argmin" => {
                            assert!(reduce.identity().len() == 2);
                            assert!(reduce.expr().len() == 1);
                            let idx = Variable::make(&format!("idx_{}", name));
                            main_stmt.push(LetStmt::make(&idx, Int::make(Dtype::I64, 0)).into());
                            let min_val = Variable::make(&format!("min_val_{}", name));
                            main_stmt.push(LetStmt::make(&min_val, &reduce.identity()[1]).into());
                            let name = Variable::make(&format!("{}", name));
                            main_stmt.push(
                                StoreStmt::make(&name, &indices, Int::make(Dtype::I64, 0)).into()
                            );
                            let mut body: Vec<Stmt> = vec![];
                            let cond = Lt::make(&reduce.expr()[0], &min_val);
                            let then_case = Seq::make([
                                Stmt::AssignStmt(AssignStmt::make(min_val, &reduce.expr()[0])),
                                Stmt::StoreStmt(StoreStmt::make(&name, &indices, &idx)),
                            ]);
                            body.push(IfThenElse::make(cond, then_case, Stmt::None).into());
                            body.push(InplaceAdd::make(&idx, Int::make(Dtype::I64, 1)).into());
                            build_nested_for(iter_vars, Seq::make(body))
                        }
                        "argmax" => {
                            assert!(reduce.identity().len() == 2);
                            assert!(reduce.expr().len() == 1);
                            let idx = Variable::make(&format!("idx_{}", name));
                            main_stmt.push(LetStmt::make(&idx, Int::make(Dtype::I64, 0)).into());
                            let max_val = Variable::make(&format!("max_val_{}", name));
                            main_stmt.push(LetStmt::make(&max_val, &reduce.identity()[1]).into());
                            let name = Variable::make(&format!("{}", name));
                            main_stmt.push(
                                StoreStmt::make(&name, &indices, Int::make(Dtype::I64, 0)).into()
                            );
                            let mut body: Vec<Stmt> = vec![];
                            let cond = Gt::make(&reduce.expr()[0], &max_val);
                            let then_case = Seq::make([
                                Stmt::AssignStmt(AssignStmt::make(max_val, &reduce.expr()[0])),
                                Stmt::StoreStmt(StoreStmt::make(&name, &indices, &idx)),
                            ]);
                            body.push(IfThenElse::make(cond, then_case, Stmt::None).into());
                            body.push(InplaceAdd::make(&idx, Int::make(Dtype::I64, 1)).into());
                            build_nested_for(iter_vars, Seq::make(body))
                        }
                        _ => todo!(),
                    };
                    main_stmt.push(fors);
                }
                _ => todo!(),
            }
            let loop_stmt = build_nested_for(&shape, Stmt::Seq(Seq::make(main_stmt)));
            stmts.push(loop_stmt);
        }
        stmts
    }
}

#[cfg(test)]
mod tests {
    use tensor_types::dtype::Dtype;

    use super::*;
    use crate::{
        halide::{
            exprs::{ Float, Int },
            loop_utils::reduction::{ argmax, argmin, max, min, sum },
            prime_expr::PrimeExpr,
            printer::IRPrinter,
        },
        hlir::traits::IntoVar,
    };

    #[test]
    fn test_argmax() {
        let n = Variable::make("n");
        let m = Variable::make("m");
        let a = Tensor::placeholder([&n, &m], "a");
        let _a = a.clone();
        let m_clone = m.clone();
        let c = compute([&n], "c", move |[i]| {
            argmax(
                [_a.slice([i, Variable::make("k").into()])],
                [
                    PrimeExpr::Int(Int::make(Dtype::BF16, 0)),
                    PrimeExpr::Float(Float::make(Dtype::F64, f64::NEG_INFINITY)),
                ],
                [(0, &m_clone, 1, "k")]
            )
        });
        let schedule = Schedule::create(vec![c]);
        let lowered = schedule.lower();
        for stmt in lowered {
            IRPrinter.print_stmt(stmt);
        }
    }

    #[test]
    fn test_argmin() {
        let n = Variable::make("n");
        let m = Variable::make("m");
        let a = Tensor::placeholder([&n, &m], "a");
        let _a = a.clone();
        let m_clone = m.clone();
        let c = compute([&n], "c", move |[i]| {
            argmin(
                [_a.slice([i, Variable::make("k").into()])],
                [
                    PrimeExpr::Int(Int::make(Dtype::BF16, 0)),
                    PrimeExpr::Float(Float::make(Dtype::F64, f64::INFINITY)),
                ],
                [(0, &m_clone, 1, "k")]
            )
        });
        let schedule = Schedule::create(vec![c]);
        let lowered = schedule.lower();
        for stmt in lowered {
            IRPrinter.print_stmt(stmt);
        }
    }
    #[test]
    fn test_max() {
        let n = Variable::make("n");
        let m = Variable::make("m");
        let a = Tensor::placeholder([&n, &m], "a");
        let _a = a.clone();
        let c = compute([&n], "c", move |[i]| {
            max(
                [_a.slice([i, Variable::make("k").into()])],
                [Int::make(Dtype::BF16, 0)],
                [(0, &m, 1, "k")]
            )
        });
        let schedule = Schedule::create(vec![c]);
        let lowered = schedule.lower();
        for stmt in lowered {
            IRPrinter.print_stmt(stmt);
        }
    }

    #[test]
    fn test_min() {
        let n = Variable::make("n");
        let m = Variable::make("m");
        let a = Tensor::placeholder([&n, &m], "a");
        let _a = a.clone();
        let c = compute([&n], "c", move |[i]| {
            min(
                [_a.slice([i, Variable::make("k").into()])],
                [Int::make(Dtype::BF16, 0)],
                [(0, &m, 1, "k")]
            )
        });
        let schedule = Schedule::create(vec![c]);
        let lowered = schedule.lower();
        for stmt in lowered {
            IRPrinter.print_stmt(stmt);
        }
    }
    #[test]
    fn test_sum() {
        let n = Variable::make("n");
        let m = Variable::make("m");
        let a = Tensor::placeholder([&n, &m], "a");
        let _a = a.clone();
        let c = compute([&n], "c", move |[i]| {
            sum(
                [_a.slice([i, Variable::make("k").into()])],
                [Int::make(Dtype::BF16, 0)],
                [(0, &m, 1, "k")]
            )
        });
        let schedule = Schedule::create(vec![c]);
        let lowered = schedule.lower();
        for stmt in lowered {
            IRPrinter.print_stmt(stmt);
        }
    }
}
