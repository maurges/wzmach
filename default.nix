{ pkgs ? import <nixpkgs> {}, ... }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "wzmach";
  version = "v1.1.0";

  src = ./.;
  cargoHash = "sha256-72SpFD5cZL/HoGqOR0juQDhSk9mKoEURnFdHHD7DAvo=";

  nativeBuildInputs = [
    pkgs.pkg-config
  ];

  buildInputs = [
    pkgs.dbus pkgs.udev pkgs.libinput
  ];

  meta = {
    description = "A mouse gesture engine";
    homepage = "https://github.com/d86leader/wzmach";
  };
}
