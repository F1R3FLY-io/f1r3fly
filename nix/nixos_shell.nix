{ pkgs ? import <nixpkgs> {} }:

let 
  pkgs = import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/fcc8660d359d2c582b0b148739a72cec476cfef5.tar.gz";
    }) {};

	pkgs_1 = import (builtins.fetchTarball {
        url = "https://github.com/NixOS/nixpkgs/archive/8ad5e8132c5dcf977e308e7bf5517cc6cc0bf7d8.tar.gz";
    }) {};

  jflex = pkgs.jflex;
  sbt = pkgs.sbt.override { jre = pkgs_1.jdk11; };

in (pkgs.buildFHSUserEnv {
  name = "rchain";

  targetPkgs = pkgs: with pkgs; [
		pkgs_1.rustc
		pkgs_1.cargo 
		pkgs_1.rustfmt
		pkgs_1.rust-analyzer
		pkgs_1.clippy
		sbt 
		pkgs_1.glibc 
		haskellPackages.BNFC 
		git 
		jflex
    coursier 
		pkgs_1.jdk11 
		which 
		pkgs_1.libiconv
    pkgs_1.gcc
    pkgs_1.protobuf
		];

  profile = ''
    export SBT_OPTS="-Xmx4g -Xss2m -Dsbt.supershell=false"
    alias rnode="./node/target/universal/stage/bin/rnode"
    export PATH="$PATH:/usr/bin"
    export LD_LIBRARY_PATH=""
  '';  
}).env
