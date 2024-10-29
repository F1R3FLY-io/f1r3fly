use models::rhoapi::expr::ExprInstance;
use models::rhoapi::{
    EAnd, EDiv, EEq, EGt, EGte, ELt, ELte, EMinus, EMinusMinus, EMod, EMult, ENeg, ENeq, ENot, EOr,
    EPercentPercent, EPlus, EPlusPlus, Expr, Par,
};

pub trait UnaryExpr {
    fn from_par(&self, p: Par) -> Expr;
}

pub trait BinaryExpr {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr;
}

impl UnaryExpr for ENot {
    fn from_par(&self, p: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::ENotBody(ENot { p: Some(p) })),
        }
    }
}

impl UnaryExpr for ENeg {
    fn from_par(&self, p: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::ENegBody(ENeg { p: Some(p) })),
        }
    }
}

impl BinaryExpr for EMult {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EMultBody(EMult {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EDiv {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EDivBody(EDiv {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EMod {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EModBody(EMod {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EPercentPercent {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EPercentPercentBody(EPercentPercent {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EPlus {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EPlusBody(EPlus {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EMinus {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EMinusBody(EMinus {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EPlusPlus {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EPlusPlusBody(EPlusPlus {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EMinusMinus {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EMinusMinusBody(EMinusMinus {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for ELt {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::ELtBody(ELt {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for ELte {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::ELteBody(ELte {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EGt {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EGtBody(EGt {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EGte {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EGteBody(EGte {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EEq {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EEqBody(EEq {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for ENeq {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::ENeqBody(ENeq {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EAnd {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EAndBody(EAnd {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}

impl BinaryExpr for EOr {
    fn from_pars(&self, p1: Par, p2: Par) -> Expr {
        Expr {
            expr_instance: Some(ExprInstance::EOrBody(EOr {
                p1: Some(p1),
                p2: Some(p2),
            })),
        }
    }
}
