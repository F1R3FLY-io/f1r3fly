with (import <nixpkgs> {});

let
    jdk = pkgs.jdk11;
    sbt = pkgs.sbt.override { jre = pkgs.jdk11; };
    jflex = pkgs.stdenv.mkDerivation rec {
      name = "jflex-1.7.0";
      src = pkgs.fetchurl {
        url = "https://jflex.de/release/jflex-1.7.0.tar.gz";
        sha256 = "sha256-c5ICqy/zyAeH7QFSUXjR//0Vvh8qAtZbeskUWwPH68w=";
      };
      buildInputs = [ pkgs.jdk ];
      installPhase = ''
        mkdir -p $out/bin
        mkdir -p $out/lib
        cp bin/jflex $out/bin
        cp lib/jflex-full-1.7.0.jar $out/lib
      '';
    };

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
    jdk
    zsh
    libiconv
    scala
		protobuf
  ];

    shellHook = ''
      export SBT_OPTS="-Xmx4g -Xss2m -Dsbt.supershell=false"
      alias rnode="./node/target/universal/stage/bin/rnode"
      export PATH="$PATH:/usr/bin"
  '';

  RUST_BACKTRACE = 1;
}
