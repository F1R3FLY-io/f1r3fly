// See casper/src/test/scala/coop/rchain/casper/merging/MergeNumberChannelSpec.scala

use std::collections::HashSet;

use casper::rust::rholang::runtime::RuntimeOps;
use crypto::rust::hash::blake2b512_random::Blake2b512Random;
use futures::future::join_all;
use models::rhoapi::{g_unforgeable::UnfInstance, GDeployId, GUnforgeable, Par};
use rholang::rust::interpreter::rho_runtime::RhoRuntime;
use rspace_plus_plus::rspace::hashing::blake2b256_hash::Blake2b256Hash;
use shared::rust::ByteString;

use crate::util::rholang::resources::mk_runtime_manager;

#[derive(Debug)]
struct DeployTestInfo {
    term: String,
    cost: i64,
    sig: String,
}

static rho_st: &str = r#"
new MergeableTag, stCh  in {
  @(*MergeableTag, *stCh)!(0) |

  contract @"SET"(ret, @v) = {
    for(@s <- @(*MergeableTag, *stCh)) {
      @(*MergeableTag, *stCh)!(s + v) | ret!(s, s + v)
    }
  } |

  contract @"READ"(ret) = {
    for(@s <<- @(*MergeableTag, *stCh)) {
      ret!(s)
    }
  }
}
"#;

fn rho_change(num: i64) -> String {
    format!(
        r#"
new retCh, out(`rho:io:stdout`) in {{
  out!(("Begin change", {})) |
  @"SET"!(*retCh, {}) |
  for(@old, @new_ <- retCh) {{
    out!(("Changed", old, "=>", new_))
  }}
}}
"#,
        num, num
    )
}

static rho_read: &str = r#"
new retCh, out(`rho:io:stdout`) in {
  @"READ"!(*retCh) |
  for(@s <- retCh) {
    out!(("Read st:", s))
  }
}
"#;

static rho_explore_read: &str = r#"
new return in {
  @"READ"!(*return)
}
"#;

fn par_rho(ori: &str, append_rho: &str) -> String {
    format!("{}|{}", ori, append_rho)
}

// fn make_sig(hex: &str) -> String {
//     let bv = ByteVector::from_hex(hex).unwrap();
//     ByteString::copy_from_slice(&bv)
// }

fn base_rho_seed() -> Blake2b512Random {
    let bytes: [u8; 128] = [1; 128];
    Blake2b512Random::create_from_bytes(&bytes)
}

fn unforgeable_name_seed() -> Par {
    Par::default().with_unforgeables(vec![GUnforgeable {
        unf_instance: Some(UnfInstance::GDeployIdBody(GDeployId {
            sig: base_rho_seed()
                .next()
                .into_iter()
                .map(|b| b as u8)
                .collect(),
        })),
    }])
}

async fn test_case(
    base_terms: Vec<String>,
    left_terms: Vec<DeployTestInfo>,
    right_terms: Vec<DeployTestInfo>,
    expected_rejected: HashSet<ByteString>,
    expected_final_result: i64,
) {
    let rm = mk_runtime_manager("merging-test", Some(unforgeable_name_seed())).await;
    let mut runtime = rm.spawn_runtime().await;

    let run_rholang = |terms: Vec<DeployTestInfo>, pre_state: Blake2b256Hash| async move {
        runtime.reset(&pre_state);

        let futures = terms
            .into_iter()
            .map(|deploy| {
                let term = deploy.term.clone();
                let mut runtime = runtime.clone();
                async move {
                    let runtime_ops = RuntimeOps::new(runtime.clone());
                    let eval_result = runtime.evaluate_with_term(&term).await.unwrap();
                    assert!(
                        eval_result.errors.is_empty(),
                        "{:?}\n{}",
                        eval_result.errors,
                        term
                    );

                    let num_chan_final = runtime_ops
                        .get_number_channels_data(&eval_result.mergeable)
                        .unwrap();

                    let soft_point = runtime.create_soft_checkpoint();
                    (soft_point, num_chan_final)
                }
            })
            .collect::<Vec<_>>();

        let eval_results = join_all(futures).await;
        let end_checkpoint = runtime.create_checkpoint();
        let (log_vec, num_chan_abs) = eval_results.into_iter().unzip::<_, _, Vec<_>, Vec<_>>();
        let num_chan_diffs = rm.convert_number_channels_to_diff(num_chan_abs, &pre_state);
    };
}
