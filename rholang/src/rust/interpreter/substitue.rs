use models::rhoapi::Par;

use super::accounting::_cost;
use super::accounting::costs::Cost;
use super::env::Env;
use super::errors::InterpreterError;

// See rholang/src/main/scala/coop/rchain/rholang/interpreter/Substitute.scala
pub trait SubstituteTrait<A> {
    fn substitute(&self, term: A, depth: i32, env: &Env<Par>) -> Result<A, InterpreterError>;

    fn substitute_no_sort(
        &self,
        term: A,
        depth: i32,
        env: &Env<Par>,
    ) -> Result<A, InterpreterError>;
}

#[derive(Clone)]
pub struct Substitute {
    pub cost: _cost,
}

impl<A> SubstituteTrait<A> for Substitute {
    fn substitute(&self, term: A, depth: i32, env: &Env<Par>) -> Result<A, InterpreterError> {
        todo!()
    }

    fn substitute_no_sort(
        &self,
        term: A,
        depth: i32,
        env: &Env<Par>,
    ) -> Result<A, InterpreterError> {
        todo!()
    }
}

impl Substitute {
    pub fn substitute_and_charge<A: prost::Message + Clone>(
        &self,
        term: &A,
        depth: i32,
        env: &Env<Par>,
    ) -> Result<A, InterpreterError> {
        // scala 'charge' function built in here
        match self.substitute(term.clone(), depth, env) {
            Ok(subst_term) => {
                self.cost.charge(Cost::create_from_generic(
                    subst_term.clone(),
                    "substitution".to_string(),
                ))?;
                Ok(subst_term)
            }
            Err(th) => {
                self.cost
                    .charge(Cost::create_from_generic(term.clone(), "".to_string()))?;
                Err(th)
            }
        }
    }

    pub fn substitute_no_sort_and_charge<A: prost::Message + Clone>(
        &self,
        term: &A,
        depth: i32,
        env: &Env<Par>,
    ) -> Result<A, InterpreterError> {
        // scala 'charge' function built in here
        match self.substitute_no_sort(term.clone(), depth, env) {
            Ok(subst_term) => {
                self.cost.charge(Cost::create_from_generic(
                    subst_term.clone(),
                    "substitution".to_string(),
                ))?;
                Ok(subst_term)
            }
            Err(th) => {
                self.cost
                    .charge(Cost::create_from_generic(term.clone(), "".to_string()))?;
                Err(th)
            }
        }
    }
}
