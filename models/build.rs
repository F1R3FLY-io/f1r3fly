extern crate prost_build;

// https://docs.rs/prost-build/latest/prost_build/struct.Config.html

fn main() {
    let mut prost_build: prost_build::Config = prost_build::Config::new();
    prost_build.btree_map(&["."]);

    prost_build.message_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, Eq, Hash, Ord, PartialOrd)]",
    );
    prost_build.message_attribute(".", "#[repr(C)]");

    prost_build.enum_attribute(
        ".",
        "#[derive(serde::Serialize, serde::Deserialize, std::cmp::Eq, Hash, Ord, PartialOrd)]",
    );
    prost_build.enum_attribute(".", "#[repr(C)]");

    prost_build
        .compile_protos(
            &[
                "scalapb/scalapb.proto",
                "RhoTypes.proto",
                "RSpacePlusPlusTypes.proto",
            ],
            &["src/main/protobuf/", "src/"],
        )
        .unwrap();
}
