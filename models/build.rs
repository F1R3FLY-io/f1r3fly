extern crate prost_build;

// https://docs.rs/prost-build/latest/prost_build/struct.Config.html

use std::fs;

fn main() {
    let mut prost_build: prost_build::Config = prost_build::Config::new();
    prost_build.btree_map(&["."]);

    prost_build.message_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, Eq, Ord, PartialOrd)]",
    );
    prost_build.message_attribute(".", "#[repr(C)]");

    prost_build.enum_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, Eq, Ord, PartialOrd)]",
    );
    prost_build.enum_attribute(".", "#[repr(C)]");

    prost_build
        .compile_protos(
            &[
                "scalapb/scalapb.proto",
                "RhoTypes.proto",
                "RSpacePlusPlusTypes.proto",
                "RholangScalaRustTypes.proto",
            ],
            &["src/main/protobuf/", "src/"],
        )
        .unwrap();

    // Remove PartialEq from specific generated structs
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let file_path = format!("{}/rhoapi.rs", out_dir);
    let content = fs::read_to_string(&file_path).expect("Unable to read file");

    let modified_content = content
        .lines()
        .map(|line| {
            if line.contains("#[derive(Clone, PartialEq, ::prost::Message)]")
                || line.contains("#[derive(Clone, PartialEq, ::prost::Oneof)]")
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
