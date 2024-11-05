use crate::rust::interpreter::compiler::rholang_ast::{Proc, UriLiteral};
use crate::rust::interpreter::errors::InterpreterError;
use models::rhoapi::Expr;
use models::rust::utils::{new_gbool_expr, new_gint_expr, new_gstring_expr, new_guri_expr};

/*
 This normalizer works with various types of "ground" (primitive) values, such as Bool, Int, String, and Uri.
*/
pub fn normalize_ground(proc: &Proc) -> Result<Expr, InterpreterError> {
    match proc.clone() {
        Proc::BoolLiteral { value, .. } => Ok(new_gbool_expr(value)),

        Proc::LongLiteral { value, .. } => Ok(new_gint_expr(value)),

        // The 'value' here is already stripped. This happens in custom parser.
        Proc::StringLiteral { value, .. } => Ok(new_gstring_expr(value)),

        // The 'value' here is already stripped. This happens in custom parser.
        Proc::UriLiteral(UriLiteral { value, .. }) => Ok(new_guri_expr(value)),

        _ => Err(InterpreterError::BugFoundError(format!(
            "Expected a ground type, found: {:?}",
            proc
        ))),
    }
}

/*
  In the new engine, we don't have a separate BoolMatcher normalizer for BoolLiteral,
  which is why the tests with BoolMatcherSpec as well as with GroundMatcherSpec will be described below.
 */
#[cfg(test)]
mod tests {
  use models::rhoapi::expr::ExprInstance;
  use super::*;

  //rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/BoolMatcherSpec.scala
  #[test]
  fn bool_true_should_compile_as_gbool_true() {
    let proc = Proc::BoolLiteral {
      value: true,
      line_num: 1,
      col_num: 1,
    };
    let result = normalize_ground(&proc);
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert_eq!(expr.expr_instance, Some(ExprInstance::GBool(true)));
  }

  //rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/BoolMatcherSpec.scala
  #[test]
  fn bool_false_should_compile_as_gbool_false() {
    let proc = Proc::BoolLiteral {
      value: false,
      line_num: 1,
      col_num: 1,
    };
    let result = normalize_ground(&proc);
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert_eq!(expr.expr_instance, Some(ExprInstance::GBool(false)));
  }

  //rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/GroundMatcherSpec.scala
  #[test]
  fn ground_int_should_compile_as_gint() {
    let proc = Proc::LongLiteral {
      value: 7,
      line_num: 1,
      col_num: 1,
    };
    let result = normalize_ground(&proc);
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert_eq!(expr.expr_instance, Some(ExprInstance::GInt(7)));
  }

  //rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/GroundMatcherSpec.scala
  #[test]
  fn ground_string_should_compile_as_gstring() {
    let proc = Proc::StringLiteral {
      value: "String".to_string(),
      line_num: 1,
      col_num: 1,
    };
    let result = normalize_ground(&proc);
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert_eq!(expr.expr_instance, Some(ExprInstance::GString("String".to_string())));
  }

  //rholang/src/test/scala/coop/rchain/rholang/interpreter/compiler/normalizer/GroundMatcherSpec.scala
  /*
    Steven: I'm not sure that this stripped works as in Scala, because when I provide `rho:uri` as a value,
    normalizer should return "rho:uri" not an "`rho:uri`"
   */
  #[test]
  fn ground_uri_should_compile_as_guri() {
    let proc = Proc::UriLiteral(UriLiteral {
      value: "rho:uri".to_string(),
      line_num: 1,
      col_num: 1,
    });
    let result = normalize_ground(&proc);
    assert!(result.is_ok());
    let expr = result.unwrap();
    assert_eq!(expr.expr_instance, Some(ExprInstance::GUri("rho:uri".to_string())));
  }

  #[test]
  fn unsupported_proc_should_return_bug_found_error() {
    let proc = Proc::Nil {
      line_num: 1,
      col_num: 1,
    };
    let result = normalize_ground(&proc);
    assert!(matches!(result, Err(InterpreterError::BugFoundError(_))));
  }
}


