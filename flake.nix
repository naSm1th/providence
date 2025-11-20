{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.05"; # or whatever vers
    };
  
  outputs = { self, nixpkgs, ... }@inputs:
    let
     system = "x86_64-linux"; # your version
     pkgs = nixpkgs.legacyPackages.${system};    
    in
    {
      devShells.${system}.default = pkgs.mkShell
      {
        packages = with pkgs; [ 
            rustup
            dbus
       ]; # whatever you need
      };
    };
}
