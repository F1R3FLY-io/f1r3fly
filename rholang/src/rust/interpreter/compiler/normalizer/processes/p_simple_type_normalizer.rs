use crate::rust::interpreter::compiler::normalize::{ProcVisitInputs, ProcVisitOutputs};
use crate::rust::interpreter::compiler::rholang_ast::SimpleType;
use crate::rust::interpreter::errors::InterpreterError;
use crate::rust::interpreter::util::prepend_connective;
use models::rhoapi::connective::ConnectiveInstance;
use models::rhoapi::Connective;

pub fn normalize_simple_type(
    simple_type: &SimpleType,
    input: ProcVisitInputs,
) -> Result<ProcVisitOutputs, InterpreterError> {
    let connective_instance = match simple_type {
        SimpleType::Bool { .. } => ConnectiveInstance::ConnBool(true),
        SimpleType::Int { .. } => ConnectiveInstance::ConnInt(true),
        SimpleType::String { .. } => ConnectiveInstance::ConnString(true),
        SimpleType::Uri { .. } => ConnectiveInstance::ConnUri(true),
        SimpleType::ByteArray { .. } => ConnectiveInstance::ConnByteArray(true),
    };

    let connective = Connective {
        connective_instance: Some(connective_instance),
    };

    Ok(ProcVisitOutputs {
        par: {
            let mut updated_par = prepend_connective(
                input.par.clone(),
                connective,
                input.bound_map_chain.depth() as i32,
            );
            updated_par.connective_used = true;
            updated_par
        },
        free_map: input.free_map,
    })
}

//rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/ProcMatcherSpec.scala
#[cfg(test)]
mod tests {
    use crate::rust::interpreter::compiler::normalize::normalize_match_proc;
    use crate::rust::interpreter::compiler::rholang_ast::{Proc, SimpleType};
    use crate::rust::interpreter::test_utils::utils::proc_visit_inputs_and_env;
    use models::rhoapi::connective::ConnectiveInstance::{
        ConnBool, ConnByteArray, ConnInt, ConnString, ConnUri,
    };

    use models::rhoapi::{Connective, Par};
    use pretty_assertions::assert_eq;

    #[test]
    fn p_simple_type_should_result_in_a_connective_of_the_correct_type() {
        let (inputs, env) = proc_visit_inputs_and_env();
        let proc_bool = Proc::SimpleType(SimpleType::new_bool());
        let proc_int = Proc::SimpleType(SimpleType::new_int());
        let proc_string = Proc::SimpleType(SimpleType::new_string());
        let proc_uri = Proc::SimpleType(SimpleType::new_uri());
        let proc_byte_array = Proc::SimpleType(SimpleType::new_bytearray());

        let result_bool = normalize_match_proc(&proc_bool, inputs.clone(), &env);
        let result_int = normalize_match_proc(&proc_int, inputs.clone(), &env);
        let result_string = normalize_match_proc(&proc_string, inputs.clone(), &env);
        let result_uri = normalize_match_proc(&proc_uri, inputs.clone(), &env);
        let result_byte_array = normalize_match_proc(&proc_byte_array, inputs.clone(), &env);

        assert_eq!(
            result_bool.unwrap().par,
            Par {
                connectives: vec![Connective {
                    connective_instance: Some(ConnBool(true))
                }],
                connective_used: true,
                ..Par::default().clone()
            }
        );

        assert_eq!(
            result_int.unwrap().par,
            Par {
                connectives: vec![Connective {
                    connective_instance: Some(ConnInt(true))
                }],
                connective_used: true,
                ..Par::default().clone()
            }
        );

        assert_eq!(
            result_string.unwrap().par,
            Par {
                connectives: vec![Connective {
                    connective_instance: Some(ConnString(true))
                }],
                connective_used: true,
                ..Par::default().clone()
            }
        );

        assert_eq!(
            result_uri.unwrap().par,
            Par {
                connectives: vec![Connective {
                    connective_instance: Some(ConnUri(true))
                }],
                connective_used: true,
                ..Par::default().clone()
            }
        );

        assert_eq!(
            result_byte_array.unwrap().par,
            Par {
                connectives: vec![Connective {
                    connective_instance: Some(ConnByteArray(true))
                }],
                connective_used: true,
                ..Par::default().clone()
            }
        );
    }
}
