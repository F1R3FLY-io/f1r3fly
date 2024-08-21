use models::{
    rhoapi::{expr::ExprInstance, Expr, GDeployerId, Par},
    rust::utils::{single_expr, single_unforgeable},
};

pub struct RhoByteArray;
use models::rhoapi::g_unforgeable::UnfInstance;
use models::rhoapi::GPrivate;
use models::rhoapi::GSysAuthToken;
use models::rhoapi::GUnforgeable;

impl RhoByteArray {
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

    pub fn create_par(bytes: Vec<u8>) -> Par {
        Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::GByteArray(bytes)),
        }])
    }
}

pub struct RhoString;

impl RhoString {
    pub fn unapply(p: &Par) -> Option<String> {
        if let Some(expr) = single_expr(p) {
            if let Expr {
                expr_instance: Some(ExprInstance::GString(str)),
            } = expr
            {
                return Some(str);
            }
        }
        None
    }

    pub fn create_par(s: String) -> Par {
        Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::GString(s)),
        }])
    }
}

pub struct RhoBoolean;

impl RhoBoolean {
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

pub struct RhoNumber;

impl RhoNumber {
    pub fn create_expr(i: i64) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::GInt(i)),
        }
    }

    pub fn unapply(p: Par) -> Option<i64> {
        if let Some(expr) = single_expr(&p) {
            if let Expr {
                expr_instance: Some(ExprInstance::GInt(v)),
            } = expr
            {
                return Some(v);
            }
        }
        None
    }
}

pub struct RhoUri;

impl RhoUri {
    pub fn create_par(s: String) -> Par {
        Par::default().with_exprs(vec![Expr {
            expr_instance: Some(ExprInstance::GUri(s)),
        }])
    }

    pub fn unapply(p: &Par) -> Option<String> {
        if let Some(expr) = single_expr(&p) {
            if let Expr {
                expr_instance: Some(ExprInstance::GUri(s)),
            } = expr
            {
                return Some(s);
            }
        }
        None
    }
}

pub struct RhoDeployerId;

impl RhoDeployerId {
    pub fn create_par(bytes: Vec<u8>) -> Par {
        Par::default().with_unforgeables(vec![GUnforgeable {
            unf_instance: Some(UnfInstance::GDeployerIdBody(GDeployerId {
                public_key: bytes,
            })),
        }])
    }

    pub fn unapply(p: &Par) -> Option<Vec<u8>> {
        if let Some(expr) = single_unforgeable(&p) {
            if let GUnforgeable {
                unf_instance: Some(UnfInstance::GDeployerIdBody(id)),
            } = expr
            {
                return Some(id.public_key);
            }
        }
        None
    }
}

pub struct RhoName;

impl RhoName {
    pub fn create_par(gprivate: GPrivate) -> Par {
        Par::default().with_unforgeables(vec![GUnforgeable {
            unf_instance: Some(UnfInstance::GPrivateBody(gprivate)),
        }])
    }

    pub fn unapply(p: &Par) -> Option<GPrivate> {
        if let Some(expr) = single_unforgeable(&p) {
            if let GUnforgeable {
                unf_instance: Some(UnfInstance::GPrivateBody(gprivate)),
            } = expr
            {
                return Some(gprivate);
            }
        }
        None
    }
}

pub struct RhoSysAuthToken;

impl RhoSysAuthToken {
    pub fn create_par(token: GSysAuthToken) -> Par {
        Par::default().with_unforgeables(vec![GUnforgeable {
            unf_instance: Some(UnfInstance::GSysAuthTokenBody(token)),
        }])
    }

    pub fn unapply(p: &Par) -> Option<GSysAuthToken> {
        if let Some(expr) = single_unforgeable(&p) {
            if let GUnforgeable {
                unf_instance: Some(UnfInstance::GSysAuthTokenBody(token)),
            } = expr
            {
                return Some(token);
            }
        }
        None
    }
}
