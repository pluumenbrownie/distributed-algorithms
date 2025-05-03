{
  description = "A Rust flake which installs a Jupyter kernel.";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    rust-overlay,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        overlays = [(import rust-overlay)];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        KernelsDir = ".jupyter/kernels";
      in {
        devShells.default = with pkgs;
          mkShell {
            buildInputs = [
              pkg-config
              rust-bin.beta.latest.default
              python312Packages.jupyter
              python312Packages.ipympl
              evcxr
            ];

            shellHook = ''
              mkdir ${KernelsDir}
              evcxr_jupyter --install > /dev/null 2>&1
              mv "$HOME/.local/share/jupyter/kernels/rust" "${KernelsDir}/rust/"
              export JUPYTER_PATH="$PWD/.jupyter"
            '';
          };
      }
    );
}
