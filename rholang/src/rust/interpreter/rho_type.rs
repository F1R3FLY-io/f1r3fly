use models::{
    rhoapi::{expr::ExprInstance, Expr, Par},
    rust::utils::single_expr,
};

pub struct ByteArray;

impl ByteArray {
    pub fn unapply(p: &Par) -> Option<Vec<u8>> {
        if let Some(expr) = single_expr(p) {
            if let Expr {
                expr_instance: Some(ExprInstance::GByteArray(bs)),
            } = expr
            {
                return Some(bs);
            }
        }
        None
    }

    pub fn apply(bytes: Vec<u8>) -> Par {
        Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::GByteArray(bytes)),
        }])
    }
}

pub struct Boolean;

impl Boolean {
    pub fn create_expr(b: bool) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::GBool(b)),
        }
    }

    pub fn unapply(p: Par) -> Option<bool> {
        if let Some(expr) = single_expr(&p) {
            if let Expr {
                expr_instance: Some(ExprInstance::GBool(b)),
            } = expr
            {
                return Some(b);
            }
        }
        None
    }
}
