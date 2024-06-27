use crate::halide::{
    prime_expr::PrimeExpr,
    ir_cmp::expr_equal,
    let_stmt::LetStmt,
    stmt::Stmt,
    traits::{ mutate_expr, IRMutateVisitor, MutatorGetSet },
};

#[derive(Clone, Debug)]
pub struct SubstituteExpr {
    find: PrimeExpr,
    replace: PrimeExpr,
    stmt: Stmt,
    expr: PrimeExpr,
}

impl SubstituteExpr {
    pub fn new() -> Self {
        SubstituteExpr {
            find: PrimeExpr::None,
            replace: PrimeExpr::None,
            stmt: Stmt::None,
            expr: PrimeExpr::None,
        }
    }

    pub fn find(&self) -> &PrimeExpr {
        &self.find
    }

    pub fn replace(&self) -> &PrimeExpr {
        &self.replace
    }

    pub fn set_find<T: Into<PrimeExpr>>(&mut self, find: T) {
        self.find = find.into();
    }

    pub fn set_replace<T: Into<PrimeExpr>>(&mut self, replace: T) {
        self.replace = replace.into();
    }
}

impl MutatorGetSet for SubstituteExpr {
    fn set_expr<T: Into<PrimeExpr>>(&mut self, expr: T) {
        self.expr = expr.into();
    }

    fn set_stmt<T: Into<Stmt>>(&mut self, stmt: T) {
        self.stmt = stmt.into();
    }

    fn expr(&self) -> &PrimeExpr {
        &self.expr
    }

    fn stmt(&self) -> &Stmt {
        &self.stmt
    }
}

impl IRMutateVisitor for SubstituteExpr {
    fn mutate_expr(&mut self, expr: &PrimeExpr) -> PrimeExpr {
        if expr_equal(&self.find, expr) {
            return self.replace.clone();
        } else {
            return mutate_expr(self, expr);
        }
    }

    fn visit_let_stmt(&mut self, let_stmt: &LetStmt) {
        let body = self.mutate_expr(let_stmt.body());
        if &body==let_stmt.body() {
            self.set_stmt(let_stmt);
        } else {
            self.set_stmt(LetStmt::make(let_stmt.var(), body));
        }
    }
}
