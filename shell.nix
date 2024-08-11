with import (fetchTarball https://github.com/NixOS/nixpkgs/archive/5e0ca22929f3342b19569b21b2f3462f053e497b.tar.gz) { };
stdenv.mkDerivation {
  name = "rustysdr";
  buildInputs = [
    rustc
    cargo
    volk
    fftw
    fftwFloat
    caddy
  ];
}
