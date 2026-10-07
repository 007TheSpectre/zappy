{ pkgs ? import <nixpkgs> {}, mcc-env }:

let
  mccShell = pkgs.mkShell.override { stdenv = mcc-env; };
in
mccShell {
  name = "zapfrites";
  buildInputs = with pkgs; [
    clang-tools
    cmake
    gnumake
    rustup
    rustc
    cargo
    rustfmt
    rust-analyzer
    python3
  ];

  shellHook = ''
    echo "Entering Nix shell: C++ + Rust (cargo) environment"
  '';
}
