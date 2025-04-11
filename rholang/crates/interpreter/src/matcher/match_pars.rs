use models::rhoapi::{
    Bundle, Connective, Expr, GUnforgeable, Match, New, Par, Receive, ReceiveBind, Send,
};

/*
 * This file is doing a custom comparison with two 'Pars'
 * It checks all the fields except 'locallyFree'
 * This is NOT handling every check of 'locallyFree' for each message in 'RhoTypes.proto'
 * Only the necessary for the 'Par, Par' trait implementation
 * This may be a problem later on. If so, probably should switch to BitSet like Scala side does
*/

pub fn compare_sends_without_locally_free(a: &Send, b: &Send) -> bool {
    a.chan == b.chan
        && a.data == b.data
        && a.persistent == b.persistent
        && a.connective_used == b.connective_used
}

pub fn compare_receives_without_locally_free(a: &Receive, b: &Receive) -> bool {
    a.binds
        .iter()
        .zip(b.binds.iter())
        .all(|(t, p)| compare_receive_binds_without_locally_free(t, p))
        && match_pars(&a.body.as_ref().unwrap(), &b.body.as_ref().unwrap())
        && a.persistent == b.persistent
        && a.peek == b.peek
        && a.bind_count == b.bind_count
        && a.connective_used == b.connective_used
}

pub fn compare_news_without_locally_free(a: &New, b: &New) -> bool {
    a.bind_count == b.bind_count
        && match_pars(&a.p.as_ref().unwrap(), &b.p.as_ref().unwrap())
        && a.uri == b.uri
        && a.injections == b.injections
}

pub fn compare_exprs_without_locally_free(a: &Expr, b: &Expr) -> bool {
    a.expr_instance == b.expr_instance
}

pub fn compare_matches_without_locally_free(a: &Match, b: &Match) -> bool {
    a.target == b.target && a.cases == b.cases && a.connective_used == b.connective_used
}

pub fn compare_unforgeables_without_locally_free(a: &GUnforgeable, b: &GUnforgeable) -> bool {
    a.unf_instance == b.unf_instance
}

pub fn compare_bundles_without_locally_free(a: &Bundle, b: &Bundle) -> bool {
    a.body == b.body && a.write_flag == b.write_flag && a.read_flag == b.read_flag
}

pub fn compare_connectives_without_locally_free(a: &Connective, b: &Connective) -> bool {
    a.connective_instance == b.connective_instance
}

pub fn compare_receive_binds_without_locally_free(a: &ReceiveBind, b: &ReceiveBind) -> bool {
    a.patterns
        .iter()
        .zip(b.patterns.iter())
        .all(|(t, p)| match_pars(t, p))
        && match_pars(&a.source.as_ref().unwrap(), &b.source.as_ref().unwrap())
        && a.remainder == b.remainder
        && a.free_count == b.free_count
}

pub fn match_pars(target: &Par, pattern: &Par) -> bool {
    target.sends.len() == pattern.sends.len()
        && target.receives.len() == pattern.receives.len()
        && target.news.len() == pattern.news.len()
        && target.exprs.len() == pattern.exprs.len()
        && target.matches.len() == pattern.matches.len()
        && target.unforgeables.len() == pattern.unforgeables.len()
        && target.bundles.len() == pattern.bundles.len()
        && target
            .sends
            .iter()
            .zip(pattern.sends.iter())
            .all(|(t, p)| compare_sends_without_locally_free(t, p))
        && target
            .receives
            .iter()
            .zip(pattern.receives.iter())
            .all(|(t, p)| compare_receives_without_locally_free(t, p))
        && target
            .news
            .iter()
            .zip(pattern.news.iter())
            .all(|(t, p)| compare_news_without_locally_free(t, p))
        && target
            .exprs
            .iter()
            .zip(pattern.exprs.iter())
            .all(|(t, p)| compare_exprs_without_locally_free(t, p))
        && target
            .matches
            .iter()
            .zip(pattern.matches.iter())
            .all(|(t, p)| compare_matches_without_locally_free(t, p))
        && target
            .unforgeables
            .iter()
            .zip(pattern.unforgeables.iter())
            .all(|(t, p)| compare_unforgeables_without_locally_free(t, p))
        && target
            .bundles
            .iter()
            .zip(pattern.bundles.iter())
            .all(|(t, p)| compare_bundles_without_locally_free(t, p))
        && target
            .connectives
            .iter()
            .zip(pattern.connectives.iter())
            .all(|(t, p)| compare_connectives_without_locally_free(t, p))
        && target.connective_used == pattern.connective_used
}
