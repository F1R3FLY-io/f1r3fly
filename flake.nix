{
  inputs = {
    typelevel-nix.url = "github:typelevel/typelevel-nix";
    nixpkgs.follows = "typelevel-nix/nixpkgs";
    oldNixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
    flake-utils.follows = "typelevel-nix/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, oldNixpkgs, flake-utils, typelevel-nix, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        graalOverlay = final: prev: rec {
          holyGraal = with oldNixpkgs.legacyPackages.${system}; graalvm17-ce.override {
            products = with graalvmCEPackages; [
              js-installable-svm-java17
              native-image-installable-svm-java17
            ];
          };
          jdk = holyGraal;
          jre = holyGraal;
        };
        ammOverlay = final: prev: {
          hematite = prev.ammonite.overrideAttrs rec {
            version = "2.5.11";
            src = builtins.fetchurl {
              url =
                "https://github.com/lihaoyi/Ammonite/releases/download/${version}/2.12-${version}";
              sha256 = "0ycwdcpprfd195i5f2ds03z2vpifv8fky6i9wh9v328z0glnjwrg";
            };
          };
        };
        overlays = [ typelevel-nix.overlay (import rust-overlay) graalOverlay ammOverlay ];
        pkgs = import nixpkgs {
          inherit system overlays;
          config.allowUnfree = true;
        };
        pkgsCross = import nixpkgs {
          inherit system overlays;
          crossSystem = nixpkgs.lib.systems.examples.aarch64-multiplatform;
        };
      in
      with pkgs;
      {
        devShells.default = devshell.mkShell {
          commands = [
            {
               name = "aarch64-linux-gnu-gcc";
               package = pkgsCross.buildPackages.gcc;
               help = "Cross-compiler for AArch64 targets";
            }
            {
              name = "docker";
              package = docker;
            }
            {
              name = "cargo";
              package = rust-bin.stable.latest.default.override {
                targets = [ "x86_64-apple-darwin" "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];
              };
              help = "The Rust package management tool";
            }
            {
              name = "rustc";
              package = rust-bin.stable.latest.default.override {
                targets = [ "x86_64-apple-darwin" "x86_64-unknown-linux-gnu" "aarch64-unknown-linux-gnu" ];
              };
              help = "The Rust compiler";
            }
            { name = "rust-analyzer";
              package = "rust-analyzer-unwrapped";
              help = "Language server for Rust";
            }
            {
              name = "bnfc";
              package = "haskellPackages.BNFC";
              help = "EBNF parser generator targeting several languages";
            }
            {
              name = "jflex";
              package = "jflex";
              help = "Java lexical analyzer generator";
            }
            {
              name = "amm";
              package = hematite;
              help = "Ammonite REPL for Scala";
            }
            {
              name = "grpcurl";
              package = grpcurl;
              help = "CURL-inspired CLI for gRPC services";
            }
            {
              name = "minikube";
              package = minikube;
              help = "Local Kubernetes cluster CLI";
            }
            {
              name = "kubectl";
              package = kubectl;
              help = "Kubernetes CLI";
            }
            {
              name = "oci";
              package = oci-cli;
              help = "Oracle Cloud CLI";
            }
            {
              name = "dhall";
              package = dhall;
              help = "Dhall configuration language";
            }
            {
              name = "dhall-to-yaml-ng";
              package = dhall-yaml;
              help = "Dhall-to-YAML utility";
            }
          ];
          imports = [ typelevel-nix.typelevelShell ];
          name = "f1r3fly-shell";
          typelevelShell = {
		        jdk.package = holyGraal;
          };
        };
      }
    );
}
