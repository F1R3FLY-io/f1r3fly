extern crate prost_build;

// https://docs.rs/prost-build/latest/prost_build/struct.Config.html

fn main() {
    let mut prost_build_rhotypes: prost_build::Config = prost_build::Config::new();
    prost_build_rhotypes.btree_map(&["."]);

    prost_build_rhotypes.message_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, Eq, Hash, Ord, PartialOrd)]",
    );
    prost_build_rhotypes.message_attribute(".", "#[repr(C)]");

    prost_build_rhotypes.enum_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, std::cmp::Eq, Hash, Ord, PartialOrd)]",
    );
    prost_build_rhotypes.enum_attribute(".", "#[repr(C)]");

    prost_build_rhotypes
        .compile_protos(&["src/scalapb/scalapb.proto", "src/protobuf/RhoTypes.proto"], &["src/"])
        .unwrap();
}
