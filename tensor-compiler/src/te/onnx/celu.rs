use std::{ panic::Location, sync::Arc };

use tensor_types::{ dtype::Dtype, type_promote::FloatOut };

use crate::{
    halide::{
        exprs::{ Load, Max, Min },
        let_stmt::LetStmt,
        prime_expr::PrimeExpr,
        stmt::Stmt,
        store_stmt::StoreStmt,
        substitute::subsititue_var::SubstituteVar,
        utils::{ dtype_one, dtype_zero, exp },
        variable::Variable,
    },
    iter_var::IterVar,
    te::{
        context::Context,
        stages::{ Body, Stage },
        strides_cal_helper::elementwise_strides_cal,
        tensor::{ StridesCal, Tensor },
    },
    to_prim_expr::ToPrimeExpr,
};

pub fn common_celu(
    is_output: bool,
    inputs: &Vec<Body>,
    shape: &Vec<PrimeExpr>,
    alpha: PrimeExpr,
    res_dtype: Dtype,
    output_id: usize
) -> Body {
    let func = |stage_out_id: usize| {
        let var: PrimeExpr = Variable::make(&format!("%{}_val", stage_out_id)).into();
        let max: PrimeExpr = Max::make(dtype_zero(res_dtype), var.clone()).into();
        let min: PrimeExpr = Min::make(
            dtype_zero(res_dtype),
            alpha.clone() * (exp(var / alpha.clone()) - dtype_one(res_dtype))
        ).into();
        max + min
    };
    if is_output {
        common_celu_helper(
            inputs,
            shape,
            output_id,
            res_dtype,
            |dims: &Vec<IterVar>, stage_out_id: usize| {
                Body::Stmt(
                    StoreStmt::make(
                        &Variable::make(&format!("%{}", output_id)),
                        dims
                            .iter()
                            .enumerate()
                            .map(
                                |(idx, x)|
                                    x.var().to_prime_expr() *
                                    Load::make(&format!("%{}.s", output_id), idx).into()
                            )
                            .reduce(|acc, x| acc + x)
                            .unwrap_or(0i64.into()),
                        func(stage_out_id)
                    ).into()
                )
            }
        )
    } else {
        common_celu_helper(inputs, shape, output_id, res_dtype, |_, stage_out_id: usize| {
            Body::Stmt(
                Stmt::LetStmt(
                    LetStmt::make(
                        &Variable::make(&format!("%{}_val", output_id)),
                        func(stage_out_id),
                        false,
                        Stmt::None
                    )
                ).into()
            )
        })
    }
}

pub fn common_celu_helper<F>(
    inputs: &Vec<Body>,
    shape: &Vec<PrimeExpr>,
    output_id: usize,
    res_dtype: Dtype,
    post: F
) -> Body
    where F: Fn(&Vec<IterVar>, usize) -> Body
{
    match &inputs[0] {
        Body::Stage(stage) => {
            let mut stage = stage.clone();
            let dims = (0..shape.len())
                .map(|x| IterVar::new(0i64, shape[x].clone(), 1i64, &format!("ax{}", x)))
                .collect::<Vec<IterVar>>();

            let mut subs_var = SubstituteVar::new();
            subs_var.add_replacement(
                Variable::new(format!("%{}.s", stage.out_id)),
                Variable::new(format!("%{}.s", output_id))
            );
            stage.bodys.push(post(&dims, stage.out_id));

            for i in stage.bodys.iter_mut() {
                i.accept_mutate(&mut subs_var);
            }
            let stage = Stage {
                dims,
                bodys: stage.bodys.clone(),
                id: output_id,
                out_id: stage.out_id,
                dtype: res_dtype,
            };
            Body::Stage(stage)
        }
        _ => panic!("input is not a stage"),
    }
}

impl Context {
    #[track_caller]
    pub fn celu(&mut self, a: &Tensor, alpha: &dyn ToPrimeExpr) -> Tensor {
        let id = self.id.borrow().clone();
        *self.id.borrow_mut() += 1;
        let shape = a.shape.clone();
        let res_dtype = a.dtype._exp();
        let alpha = alpha.to_prime_expr();
        let ret = Tensor {
            shape: a.shape.clone(),
            inputs: Arc::new(vec![a.id]),
            span: Location::caller(),
            id,
            dtype: res_dtype,
            strides_cal: Arc::new(move |prev_fn: Vec<StridesCal>| {
                elementwise_strides_cal(prev_fn[0].clone())
            }),
            body_gen: Arc::new(move |inputs: Vec<Body>, is_output: bool, id: usize| {
                let alpha = alpha.clone();
                common_celu(is_output, &inputs, &shape, alpha, res_dtype, id)
            }),
        };
        self.nodes.borrow_mut().insert(id, ret.clone());
        ret
    }
}
