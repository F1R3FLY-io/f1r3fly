with (import <nixpkgs> {});

let
#    jdk = pkgs.jdk11;
    sbt = pkgs.sbt.override { jre = pkgs.jdk11; };
    # jflex = import (fetchTarball https://github.com/NixOS/nixpkgs/archive/fcc8660d359d2c582b0b148739a72cec476cfef5.tar.gz) { };

in mkShell {
  buildInputs = with pkgs; [
    ammonite
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
    git
    ps
    which
    less
    haskellPackages.BNFC
    neovim
    locale
    jflex
    coursier
    scalafmt
    sbt 
    ssh-agents
    # jdk
    zsh
    libiconv
    scala
  ];

    shellHook = ''
      export SBT_OPTS="-Xmx4g -Xss2m -Dsbt.supershell=false"
      alias rnode="./node/target/universal/stage/bin/rnode"
      export PATH="$PATH:/usr/bin"
  '';

  RUST_BACKTRACE = 1;
}