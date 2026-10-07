{ pkgs ? import <nixpkgs> {} }:

with pkgs; clangStdenv.mkDerivation {
  name = "zapfrites";
  src = ./.;

  buildInputs = [
    clang
    cmake
    gnumake
  ];

  installPhase = ''
    mkdir -p $out/bin
    if [ -f hello ]; then mv hello $out/bin/hello; fi
  '';
}
