use hashbrown::HashMap;

use crate::halide::{
    prime_expr::PrimeExpr,
    let_stmt::LetStmt,
    stmt::Stmt,
    traits::{ mutate_expr, IRMutateVisitor, MutatorGetSet },
};

#[derive(Clone, Debug)]
pub struct SubstituteExpr {
    replace: HashMap<PrimeExpr, PrimeExpr>,
    stmt: Stmt,
    expr: PrimeExpr,
}

impl SubstituteExpr {
    pub fn new() -> Self {
        SubstituteExpr {
            replace: HashMap::new(),
            stmt: Stmt::None,
            expr: PrimeExpr::None,
        }
    }

    fn find_replacement(&self, expr: &PrimeExpr) -> Option<&PrimeExpr> {
        self.replace.get(expr)
    }

    pub fn add_replacement<T: Into<PrimeExpr>>(&mut self, find: PrimeExpr, replace: T) {
        self.replace.insert(find, replace.into());
    }
    pub fn replacements(&self) -> &HashMap<PrimeExpr, PrimeExpr> {
        &self.replace
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
        if let Some(replacement) = self.find_replacement(expr) {
            replacement.clone()
        } else {
            mutate_expr(self, expr)
        }
    }

    fn visit_let_stmt(&mut self, let_stmt: &LetStmt) {
        let value = self.mutate_expr(let_stmt.value());
        let body = self.mutate_stmt(let_stmt.body());
        if &body == let_stmt.body() && &value == let_stmt.value() {
            self.set_stmt(let_stmt);
        } else {
            self.set_stmt(LetStmt::make(let_stmt.var(), value, body));
        }
    }

    fn visit_variable(&mut self, var: &crate::halide::variable::Variable) {
        if let Some(replacement) = self.find_replacement(&var.into()) {
            self.set_expr(replacement.clone());
        } else {
            self.set_expr(var);
        }
    }
}
