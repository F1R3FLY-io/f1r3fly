// See models/src/main/scala/coop/rchain/models/ParSetTypeMapper.scala

use crate::rhoapi::ESet;

use super::par_set::ParSet;

pub struct ParSetTypeMapper;

impl ParSetTypeMapper {
    pub fn eset_to_par_set(eset: ESet) -> ParSet {
        ParSet::new(
            eset.ps,
            eset.connective_used,
            eset.locally_free,
            eset.remainder,
        )
    }

    pub fn par_set_to_eset(par_set: ParSet) -> ESet {
        ESet {
            ps: par_set.ps.sorted_pars,
            locally_free: par_set.locally_free,
            connective_used: par_set.connective_used,
            remainder: par_set.remainder,
        }
    }
}
