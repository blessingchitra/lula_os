{ pkgs ? import <nixpkgs> {} }:

let
  # we use unstable version for latest rust nightly
  unstable = import (fetchTarball "https://github.com/nixos/nixpkgs/archive/nixos-unstable.tar.gz") { };
in

pkgs.mkShell {
  buildInputs = with pkgs; [
    unstable.rustup
    
    pkgsCross.riscv64.buildPackages.gcc
    pkgsCross.riscv64.buildPackages.gdb
    pkgsCross.riscv64.buildPackages.binutils
    
    qemu
    gdb

    just
    pkg-config
    git
  ];

  RUSTC_VERSION = "nightly";
  
  shellHook = ''
    rustup default nightly
    rustup component add rust-src
    rustup target add riscv64gc-unknown-none-elf

    echo "Development environment ready!"
  '';
}