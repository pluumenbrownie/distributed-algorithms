{
  description = "Flake to load Rust with Jupyter.";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };

  outputs = {
    self,
    nixpkgs,
  }: let
    system = "x86_64-linux";
    pkgs = nixpkgs.legacyPackages.${system};
  in {
    nixpkgs.overlays = [
      (
        self: super: {
          python312Packages = super.python312Packages.override {
            overrides = pyself: pysuper: {
              lmfit = pysuper.lmfit.overrideAttrs {doCheck = false;};
            };
          };
        }
      )
    ];

    devShells.${system}.default = pkgs.mkShell {
      nativeBuildInputs = with pkgs; [
        python312Packages.jupyter
        python312Packages.ipympl

        evcxr

        typst
        typstfmt
        typstyle
      ];
    };

    doCheck = false;
  };
}
