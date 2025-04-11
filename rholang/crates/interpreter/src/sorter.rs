use indextree::NodeId;

use super::normal_forms::*;
use super::{score_tree::ScoreBuilder, sort_matcher::*};

pub struct ParSorter<'a>(&'a mut Par);

impl<'a> ParSorter<'a> {
    pub fn new(par: &'a mut Par) -> ParSorter<'a> {
        ParSorter(par)
    }

    pub fn into_score_tree<'b, Builder: ScoreBuilder>(
        sorted_par: &'b Sorted<Par>,
        score: &mut Builder,
    ) {
        let mut holes = Vec::with_capacity(10);

        let current = score.begin(PAR);
        holes.push(Hole {
            location: current,
            par: sorted_par,
        });

        loop {
            match holes.pop() {
                Some(hole) => {
                    score.focus(hole.location);
                    Self::into_score_tree_inner(hole.par, score, &mut holes);
                }
                None => break,
            }
        }
    }

    fn into_score_tree_inner<'b, B: ScoreBuilder>(
        par: &'b Par,
        score: &mut B,
        holes: &mut Vec<Hole<'b>>,
    ) {
        fn punch_hole<'c, B>(target: &'c Par, score: &mut B) -> Hole<'c>
        where
            B: ScoreBuilder,
        {
            let location = score.begin(PAR);

            let hole = Hole {
                location,
                par: target,
            };

            score.done();

            return hole;
        }

        // FIXME: is there a way to eliminate this duplication?

        par.sends.iter().for_each(|send| {
            score.begin(SEND);

            score.leaf_bool(send.persistent);
            holes.push(punch_hole(&send.chan, score));
            holes.extend(send.data.iter().map(|par| punch_hole(par, score)));

            score.done();
        });
        par.receives.iter().for_each(|receive| {
            score.begin(RECEIVE);

            score.leaf_bool(receive.persistent);
            score.leaf_bool(receive.peek);

            receive.binds.iter().for_each(|bind| {
                holes.push(punch_hole(&bind.source, score));
                holes.extend(
                    bind.patterns
                        .iter()
                        .map(|pattern| punch_hole(pattern, score)),
                );

                score.done();
            });

            holes.push(punch_hole(&receive.body, score));
        });
        par.exprs.iter().for_each(|expr| {
            match expr {
                Expr::ENeg(body) => {
                    score.begin(ENEG);
                    holes.push(punch_hole(body, score));
                }
                Expr::EVar(var) => {
                    score.begin(EVAR);
                    var_score(*var, score);
                }
                Expr::ENot(body) => {
                    score.begin(ENOT);
                    holes.push(punch_hole(body, score));
                }
                Expr::EMult(p1, p2) => {
                    score.begin(EMULT);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EDiv(p1, p2) => {
                    score.begin(EDIV);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EMod(p1, p2) => {
                    score.begin(EMOD);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EPlus(p1, p2) => {
                    score.begin(EPLUS);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EMinus(p1, p2) => {
                    score.begin(EMINUS);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::ELt(p1, p2) => {
                    score.begin(ELT);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::ELte(p1, p2) => {
                    score.begin(ELTE);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EGt(p1, p2) => {
                    score.begin(EGT);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EGte(p1, p2) => {
                    score.begin(EGTE);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EEq(p1, p2) => {
                    score.begin(EEQ);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::ENeq(p1, p2) => {
                    score.begin(ENEQ);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EAnd(p1, p2) => {
                    score.begin(EAND);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EOr(p1, p2) => {
                    score.begin(EOR);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EMatches(body) => {
                    score.begin(EMATCHES);
                    holes.push(punch_hole(&body.target, score));
                    holes.push(punch_hole(&body.pattern, score));
                }
                Expr::EPercentPercent(p1, p2) => {
                    score.begin(EPERCENT);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EPlusPlus(p1, p2) => {
                    score.begin(EPLUSPLUS);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EMinusMinus(p1, p2) => {
                    score.begin(EMINUSMINUS);
                    holes.push(punch_hole(p1, score));
                    holes.push(punch_hole(p2, score));
                }
                Expr::EMap(body) => {
                    score.begin(EMAP);

                    body.ps.iter().for_each(|(k, v)| {
                        k.graft_into(score);
                        holes.push(punch_hole(v, score));
                    });
                    remainder_score(body.remainder, -1, score);
                    score.leaf_bool(body.connective_used);
                }
                Expr::ESet(body) => {
                    score.begin(ESET);

                    body.ps.iter().for_each(|par| {
                        par.graft_into(score);
                    });
                    remainder_score(body.remainder, -1, score);
                    score.leaf_bool(body.connective_used);
                }
                Expr::EList(body) => {
                    score.begin(ELIST);

                    holes.extend(body.ps.iter().map(|par| punch_hole(par, score)));
                    remainder_score(body.remainder, -1, score);
                    score.leaf_bool(body.connective_used);
                }
                Expr::ETuple(body) => {
                    score.begin(ETUPLE);

                    holes.extend(body.ps.iter().map(|par| punch_hole(par, score)));
                    score.leaf_bool(body.connective_used);
                }
                Expr::EMethod(method) => {
                    score.begin(EMETHOD);

                    score.leaf_string(method.method_name.clone());
                    holes.push(punch_hole(&method.target, score));

                    holes.extend(method.arguments.iter().map(|arg| punch_hole(arg, score)));

                    score.leaf_bool(method.connective_used);
                }

                Expr::GBool(value) => {
                    score.begin(GBOOL);
                    score.leaf_int(if *value { 0 } else { 1 });
                }
                Expr::GInt(value) => {
                    score.begin(GINT);
                    score.leaf_int(*value);
                }
                Expr::GString(value) => {
                    score.begin(GSTRING);
                    score.leaf_string(value.to_owned());
                }
                Expr::GUri(value) => {
                    score.begin(GURI);
                    score.leaf_string(value.to_owned());
                }
                Expr::GByteArray(value) => {
                    score.begin(EBYTEARR);
                    score.leaf_bytes(value.to_owned());
                }
            }
            score.done()
        });
        par.news.iter().for_each(|new| {
            score.begin(NEW);
            score.leaf_int(new.bind_count.into());

            if new.uris.is_empty() {
                score.leaf_int(0);
            } else {
                new.uris.iter().for_each(|uri| {
                    score.leaf_string(uri.to_owned());
                });
            }

            holes.push(punch_hole(&new.p, score));
            score.done();
        });
        par.matches.iter().for_each(|match_| {
            score.begin(MATCH);

            match_.cases.iter().for_each(|case| {
                holes.push(punch_hole(&case.pattern, score));
                holes.push(punch_hole(&case.source, score));
                score.leaf_int(case.free_count.into());
            });

            score.leaf_bool(match_.connective_used);

            score.done();
        });
        par.bundles.iter().for_each(|bundle| {
            score.begin(BundleSorter::bundle_score(bundle));
            holes.push(punch_hole(&bundle.body, score));
            score.done();
        });
        par.connectives.iter().for_each(|connective| {
            match connective {
                Connective::ConnAnd(body) => {
                    score.begin(CONNECTIVE_AND);
                    holes.extend(body.iter().map(|par| punch_hole(par, score)));
                }
                Connective::ConnOr(body) => {
                    score.begin(CONNECTIVE_OR);
                    holes.extend(body.iter().map(|par| punch_hole(par, score)));
                }
                Connective::ConnNot(body) => {
                    score.begin(CONNECTIVE_NOT);
                    holes.push(punch_hole(body, score));
                }
                Connective::VarRef(VarRef { index, depth }) => {
                    score.begin(CONNECTIVE_VARREF);
                    score.leaf_int((*index).into());
                    score.leaf_int((*depth).into());
                }
                Connective::ConnBool => {
                    score.begin(CONNECTIVE_BOOL);
                    score.leaf_int(1);
                }
                Connective::ConnInt => {
                    score.begin(CONNECTIVE_INT);
                    score.leaf_int(1);
                }
                Connective::ConnString => {
                    score.begin(CONNECTIVE_STRING);
                    score.leaf_int(1);
                }
                Connective::ConnUri => {
                    score.begin(CONNECTIVE_URI);
                    score.leaf_int(1);
                }
                Connective::ConnByteArray => {
                    score.begin(CONNECTIVE_BYTEARRAY);
                    score.leaf_int(1);
                }
            }
            score.done();
        });
        par.unforgeables
            .iter()
            .for_each(|unf| unforgeable_score(unf, score));
    }
}

impl<'a> SortMatcher for ParSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        score.begin(PAR);

        sort_vec(&mut self.0.sends, score);
        sort_vec(&mut self.0.receives, score);
        sort_vec(&mut self.0.exprs, score);
        sort_vec(&mut self.0.news, score);
        sort_vec(&mut self.0.matches, score);
        sort_vec(&mut self.0.bundles, score);
        sort_vec(&mut self.0.connectives, score);

        // unforageables are just bytes so we can sort it directly
        // (unforgeables compare as unsigned, so this is consistent with its score tree)
        self.0.unforgeables.sort();
        self.0
            .unforgeables
            .iter()
            .for_each(|unf| unforgeable_score(unf, score));

        score.leaf_bool(self.0.connective_used);

        score.done();
    }
}

pub struct SendSorter<'a>(&'a mut Send);

impl<'a> SendSorter<'a> {
    pub fn new(send: &'a mut Send) -> SendSorter<'a> {
        SendSorter(send)
    }
}

impl<'a> SortMatcher for SendSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        score.begin(SEND);

        score.leaf_bool(self.0.persistent);
        ParSorter(&mut self.0.chan).sort_match(score);
        self.0
            .data
            .iter_mut()
            .for_each(|par| ParSorter(par).sort_match(score));

        score.done();
    }
}

pub struct ReceiveBindSorter<'a>(&'a mut ReceiveBind);

impl<'a> ReceiveBindSorter<'a> {
    pub fn new(receive_bind: &'a mut ReceiveBind) -> ReceiveBindSorter<'a> {
        ReceiveBindSorter(receive_bind)
    }
}

impl<'a> SortMatcher for ReceiveBindSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        ParSorter(&mut self.0.source).sort_match(score);
        self.0
            .patterns
            .iter_mut()
            .for_each(|pattern| ParSorter(pattern).sort_match(score));
        remainder_score(self.0.remainder, 0, score)
    }
}

pub struct ReceiveSorter<'a>(&'a mut Receive);

impl<'a> ReceiveSorter<'a> {
    pub fn new(receive: &'a mut Receive) -> ReceiveSorter<'a> {
        ReceiveSorter(receive)
    }
}

impl<'a> SortMatcher for ReceiveSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        score.begin(RECEIVE);

        score.leaf_bool(self.0.persistent);
        score.leaf_bool(self.0.peek);

        self.0.binds.iter_mut().for_each(|bind| {
            ParSorter(&mut bind.source).sort_match(score);
            bind.patterns
                .iter_mut()
                .for_each(|pattern| ParSorter(pattern).sort_match(score));
            remainder_score(bind.remainder, 0, score)
        });

        ParSorter(&mut self.0.body).sort_match(score);

        score.leaf_int(self.0.bind_count as i64);
        score.leaf_bool(self.0.connective_used);

        score.done();
    }
}

pub struct BundleSorter<'a> {
    b: &'a mut Bundle,
}

impl<'a> BundleSorter<'a> {
    pub fn new(bundle: &'a mut Bundle) -> BundleSorter<'a> {
        BundleSorter { b: bundle }
    }

    fn bundle_score(b: &Bundle) -> i32 {
        if b.write_flag {
            if b.read_flag {
                BUNDLE_READ_WRITE
            } else {
                BUNDLE_WRITE
            }
        } else {
            if b.read_flag {
                BUNDLE_READ
            } else {
                BUNDLE_EQUIV
            }
        }
    }
}

impl<'a> SortMatcher for BundleSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        score.begin(Self::bundle_score(self.b));

        ParSorter(&mut self.b.body).sort_match(score);
        score.done();
    }
}

pub struct NewSorter<'a> {
    new: &'a mut New,
}

impl<'a> NewSorter<'a> {
    pub fn new(new: &'a mut New) -> NewSorter<'a> {
        NewSorter { new }
    }
}

impl<'a> SortMatcher for NewSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        score.begin(NEW);
        score.leaf_int(self.new.bind_count.into());

        if self.new.uris.is_empty() {
            score.leaf_int(0);
        } else {
            self.new.uris.sort();
            self.new.uris.iter().for_each(|uri| {
                score.leaf_string(uri.to_owned());
            });
        }

        ParSorter(&mut self.new.p).sort_match(score);
        score.done();
    }
}

pub struct MatchSorter<'a> {
    m: &'a mut Match,
}

impl<'a> MatchSorter<'a> {
    pub fn new(match_: &'a mut Match) -> MatchSorter<'a> {
        MatchSorter { m: match_ }
    }
}

impl<'a> SortMatcher for MatchSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        score.begin(MATCH);

        self.m.cases.iter_mut().for_each(|case| {
            ParSorter(&mut case.pattern).sort_match(score);
            ParSorter(&mut case.source).sort_match(score);
            score.leaf_int(case.free_count.into());
        });

        score.leaf_bool(self.m.connective_used);

        score.done();
    }
}

pub struct ConnectiveSorter<'a> {
    c: &'a mut Connective,
}

impl<'a> ConnectiveSorter<'a> {
    pub fn new(connective: &'a mut Connective) -> ConnectiveSorter<'a> {
        ConnectiveSorter { c: connective }
    }
}

impl<'a> SortMatcher for ConnectiveSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        match self.c {
            Connective::ConnAnd(body) => {
                score.begin(CONNECTIVE_AND);
                for par in body {
                    ParSorter(par).sort_match(score);
                }
            }
            Connective::ConnOr(body) => {
                score.begin(CONNECTIVE_OR);
                for par in body {
                    ParSorter(par).sort_match(score);
                }
            }
            Connective::ConnNot(body) => {
                score.begin(CONNECTIVE_NOT);
                ParSorter(body).sort_match(score);
            }
            Connective::VarRef(VarRef { index, depth }) => {
                score.begin(CONNECTIVE_VARREF);
                score.leaf_int((*index).into());
                score.leaf_int((*depth).into());
            }
            Connective::ConnBool => {
                score.begin(CONNECTIVE_BOOL);
                score.leaf_int(1);
            }
            Connective::ConnInt => {
                score.begin(CONNECTIVE_INT);
                score.leaf_int(1);
            }
            Connective::ConnString => {
                score.begin(CONNECTIVE_STRING);
                score.leaf_int(1);
            }
            Connective::ConnUri => {
                score.begin(CONNECTIVE_URI);
                score.leaf_int(1);
            }
            Connective::ConnByteArray => {
                score.begin(CONNECTIVE_BYTEARRAY);
                score.leaf_int(1);
            }
        }
        score.done();
    }
}

pub struct ExprSorter<'a> {
    e: &'a mut Expr,
}

impl<'a> ExprSorter<'a> {
    pub fn new(expr: &'a mut Expr) -> ExprSorter<'a> {
        ExprSorter { e: expr }
    }
}

impl<'a> SortMatcher for ExprSorter<'a> {
    fn sort_match<Builder: ScoreBuilder>(self, score: &mut Builder) {
        match self.e {
            Expr::ENeg(body) => {
                score.begin(ENEG);
                ParSorter(body).sort_match(score);
            }
            Expr::EVar(var) => {
                score.begin(EVAR);
                var_score(*var, score);
            }
            Expr::ENot(body) => {
                score.begin(ENOT);
                ParSorter(body).sort_match(score);
            }
            Expr::EMult(p1, p2) => {
                score.begin(EMULT);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EDiv(p1, p2) => {
                score.begin(EDIV);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EMod(p1, p2) => {
                score.begin(EMOD);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EPlus(p1, p2) => {
                score.begin(EPLUS);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EMinus(p1, p2) => {
                score.begin(EMINUS);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::ELt(p1, p2) => {
                score.begin(ELT);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::ELte(p1, p2) => {
                score.begin(ELTE);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EGt(p1, p2) => {
                score.begin(EGT);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EGte(p1, p2) => {
                score.begin(EGTE);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EEq(p1, p2) => {
                score.begin(EEQ);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::ENeq(p1, p2) => {
                score.begin(ENEQ);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EAnd(p1, p2) => {
                score.begin(EAND);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EOr(p1, p2) => {
                score.begin(EOR);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EMatches(body) => {
                score.begin(EMATCHES);
                ParSorter(&mut body.target).sort_match(score);
                ParSorter(&mut body.pattern).sort_match(score);
            }
            Expr::EPercentPercent(p1, p2) => {
                score.begin(EPERCENT);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EPlusPlus(p1, p2) => {
                score.begin(EPLUSPLUS);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EMinusMinus(p1, p2) => {
                score.begin(EMINUSMINUS);
                ParSorter(p1).sort_match(score);
                ParSorter(p2).sort_match(score);
            }
            Expr::EMap(body) => {
                score.begin(EMAP);

                body.ps.iter().for_each(|(k, v)| {
                    k.graft_into(score);
                    ParSorter::into_score_tree(v, score)
                });
                remainder_score(body.remainder, -1, score);
                score.leaf_bool(body.connective_used);
            }
            Expr::ESet(body) => {
                score.begin(ESET);

                body.ps.iter().for_each(|par| {
                    par.graft_into(score);
                });
                remainder_score(body.remainder, -1, score);
                score.leaf_bool(body.connective_used);
            }
            Expr::EList(body) => {
                score.begin(ELIST);

                body.ps.iter_mut().for_each(|par| {
                    ParSorter::new(par).sort_match(score);
                });
                remainder_score(body.remainder, -1, score);
                score.leaf_bool(body.connective_used);
            }
            Expr::ETuple(body) => {
                score.begin(ETUPLE);

                body.ps.iter_mut().for_each(|par| {
                    ParSorter::new(par).sort_match(score);
                });
                score.leaf_bool(body.connective_used);
            }
            Expr::EMethod(method) => {
                score.begin(EMETHOD);

                score.leaf_string(method.method_name.clone());
                ParSorter::new(&mut method.target).sort_match(score);

                method
                    .arguments
                    .iter_mut()
                    .for_each(|par| ParSorter(par).sort_match(score));

                score.leaf_bool(method.connective_used);
            }

            Expr::GBool(value) => {
                score.begin(GBOOL);
                score.leaf_int(if *value { 0 } else { 1 });
            }
            Expr::GInt(value) => {
                score.begin(GINT);
                score.leaf_int(*value);
            }
            Expr::GString(value) => {
                score.begin(GSTRING);
                score.leaf_string(value.to_owned());
            }
            Expr::GUri(value) => {
                score.begin(GURI);
                score.leaf_string(value.to_owned());
            }
            Expr::GByteArray(value) => {
                score.begin(EBYTEARR);
                score.leaf_bytes(value.to_owned());
            }
        }
        score.done();
    }
}

fn var_score<Builder: ScoreBuilder>(var: Var, score: &mut Builder) {
    match var {
        Var::BoundVar(level) => {
            score.begin(BOUND_VAR);
            score.leaf_int(level.into());
            score.done();
        }
        Var::FreeVar(level) => {
            score.begin(FREE_VAR);
            score.leaf_int(level.into());
            score.done();
        }
        Var::Wildcard => {
            score.leaf_int(WILDCARD.into());
        }
    }
}

fn unforgeable_score<Builder: ScoreBuilder>(unf: &GUnforgeable, score: &mut Builder) {
    score.begin(PRIVATE);
    score.leaf_bytes(unf.to_bytes_vec());
    score.done();
}

fn remainder_score<Builder: ScoreBuilder>(
    remainder: Option<Var>,
    default: i64,
    score: &mut Builder,
) {
    match remainder {
        Some(remainder) => var_score(remainder, score),
        None => {
            score.leaf_int(default);
        }
    }
}

fn sort_vec<T: Sortable, B: ScoreBuilder>(sortables: &mut Vec<T>, score: &mut B) {
    match sortables.len() {
        0 => {}
        1 => {
            sortables.first_mut().unwrap().sorter().sort_match(score);
        }
        _ => {
            let mut temp = Vec::with_capacity(sortables.len());
            sortables.drain(..).for_each(|mut t| {
                let mut forked = score.fork();
                t.sorter().sort_match(&mut forked);
                temp.push((t, forked.to_tree()));
            });

            temp.sort_by(|(_, this), (_, that)| this.cmp(that));

            for (t, tree) in temp {
                sortables.push(t);
                score.attach(tree);
            }
        }
    }
}

#[derive(Clone, Copy)]
struct Hole<'a> {
    location: NodeId,
    par: &'a Par,
}

const GBOOL: i32 = 1;
const GINT: i32 = 2;
const GSTRING: i32 = 3;
const GURI: i32 = 4;
const PRIVATE: i32 = 5;

const ELIST: i32 = 6;
const ETUPLE: i32 = 7;
const ESET: i32 = 8;
const EMAP: i32 = 9;

const BOUND_VAR: i32 = 50;
const FREE_VAR: i32 = 51;
const WILDCARD: i32 = 52;

const EVAR: i32 = 100;
const ENEG: i32 = 101;
const EMULT: i32 = 102;
const EDIV: i32 = 103;
const EPLUS: i32 = 104;
const EMINUS: i32 = 105;
const ELT: i32 = 106;
const ELTE: i32 = 107;
const EGT: i32 = 108;
const EGTE: i32 = 109;
const EEQ: i32 = 110;
const ENEQ: i32 = 111;
const ENOT: i32 = 112;
const EAND: i32 = 113;
const EOR: i32 = 114;
const EMETHOD: i32 = 115;
const EBYTEARR: i32 = 116;
const EMATCHES: i32 = 118;
const EPERCENT: i32 = 119;
const EPLUSPLUS: i32 = 120;
const EMINUSMINUS: i32 = 121;
const EMOD: i32 = 122;

const SEND: i32 = 300;
const RECEIVE: i32 = 301;
const NEW: i32 = 303;
const MATCH: i32 = 304;

const BUNDLE_EQUIV: i32 = 305;
const BUNDLE_READ: i32 = 306;
const BUNDLE_WRITE: i32 = 307;
const BUNDLE_READ_WRITE: i32 = 308;

const CONNECTIVE_NOT: i32 = 400;
const CONNECTIVE_AND: i32 = 401;
const CONNECTIVE_OR: i32 = 402;
const CONNECTIVE_VARREF: i32 = 403;
const CONNECTIVE_BOOL: i32 = 404;
const CONNECTIVE_INT: i32 = 405;
const CONNECTIVE_STRING: i32 = 406;
const CONNECTIVE_URI: i32 = 407;
const CONNECTIVE_BYTEARRAY: i32 = 408;

const PAR: i32 = 999;
