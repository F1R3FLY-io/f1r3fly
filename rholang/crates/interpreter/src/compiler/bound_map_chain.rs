use std::ops::{Deref, DerefMut};

use smallvec::SmallVec;

use super::{bound_map::BoundContext, exports::*};

#[derive(Debug)]
pub struct BoundMapChain {
    chain: SmallVec<[BoundMap; 1]>,
}

impl BoundMapChain {
    pub fn new() -> Self {
        BoundMapChain {
            chain: SmallVec::from_buf([BoundMap::new()]),
        }
    }

    pub fn find(&self, name: &str) -> Option<((u32, &Context<BoundContext>), u32)> {
        self.chain
            .iter()
            .rev()
            .enumerate()
            .find_map(|(depth, map)| map.get(name).map(|context| (context, depth as u32)))
    }

    pub fn push(&mut self) {
        self.chain.push(BoundMap::new());
    }

    pub fn pop(&mut self) {
        self.chain.pop().expect("BoundMapChain invariant");
    }

    #[inline]
    pub fn descend<F, R>(&mut self, mut f: F) -> R
    where
        F: FnMut(&mut Self) -> R,
    {
        self.push();
        let r = f(self);
        self.pop();

        return r;
    }

    pub(crate) fn depth(&self) -> u32 {
        self.chain.len() as u32 - 1
    }

    #[inline]
    pub fn extend<F, R>(&mut self, free_map: FreeMap, mut f: F) -> R
    where
        F: FnMut(u32, &mut Self) -> R,
    {
        let bound_count = free_map.count_no_wildcards();
        self.absorb_free(free_map);

        let r = f(bound_count, self);

        self.shift(bound_count as usize);
        return r;
    }
}

impl Default for BoundMapChain {
    fn default() -> Self {
        Self::new()
    }
}

impl Deref for BoundMapChain {
    type Target = BoundMap;

    fn deref(&self) -> &Self::Target {
        self.chain.last().expect("BoundMapChain invariant")
    }
}

impl DerefMut for BoundMapChain {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.chain.last_mut().expect("BoundMapChain invariant")
    }
}
