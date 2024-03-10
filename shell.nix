with import <nixpkgs> { };
stdenv.mkDerivation {
  name = "rustysdr";
  buildInputs = [
    rustc
    cargo
    volk
    fftw
    caddy
  ];
}
