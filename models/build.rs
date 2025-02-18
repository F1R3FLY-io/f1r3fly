extern crate tonic_build;

// https://docs.rs/prost-build/latest/prost_build/struct.Config.html
// https://docs.rs/tonic-build/latest/tonic_build/struct.Builder.html#

use std::fs;

fn main() {
    tonic_build::configure()
        .build_client(true)
        .build_server(false)
        .btree_map(&["."])
        .message_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .message_attribute(".rhoapi", "#[derive(Eq, Ord, PartialOrd)]")
        .message_attribute(".rhoapi", "#[repr(C)]")
        .enum_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .enum_attribute(".rhoapi", "#[derive(Eq, Ord, PartialOrd)]")
        .enum_attribute(".rhoapi", "#[repr(C)]")
        .compile_protos(
            &[
                "CasperMessage.proto",
                "DeployServiceCommon.proto",
                "DeployServiceV1.proto",
                "ProposeServiceCommon.proto",
                "ProposeServiceV1.proto",
                "RholangScalaRustTypes.proto",
                "RhoTypes.proto",
                "RSpacePlusPlusTypes.proto",
                "ServiceError.proto",
                "scalapb/scalapb.proto",
            ],
            &["src/", "src/main/protobuf/"],
        )
        .expect("Failed to compile proto files");

    // Remove PartialEq from specific generated structs from rhoapi.rs
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let file_path = format!("{}/rhoapi.rs", out_dir);
    let content = fs::read_to_string(&file_path).expect("Unable to read file");

    let modified_content = content
        .lines()
        .map(|line| {
            if line.contains("#[derive(Clone, PartialEq, ::prost::Message)]")
                || line.contains("#[derive(Clone, PartialEq, ::prost::Oneof)]")
                || line.contains("#[derive(Clone, Copy, PartialEq, ::prost::Message)]")
                || line.contains("#[derive(Clone, Copy, PartialEq, ::prost::Oneof)]")
            {
                line.replace("PartialEq,", "")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<String>>()
        .join("\n");

    fs::write(file_path, modified_content).expect("Unable to write file");
}
