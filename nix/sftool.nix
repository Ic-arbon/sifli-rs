{ lib
, rustPlatform
, fetchFromGitHub
, pkg-config
, stdenv
, udev
}:

rustPlatform.buildRustPackage rec {
  pname = "sftool";
  version = "0.1.14";

  src = fetchFromGitHub {
    owner = "OpenSiFli";
    repo = "sftool";
    rev = version;
    hash = "sha256-xheGgtE9hZVNa4ceqQCrfiYJwlIuXm00J//0VeZ/afE=";
  };

  cargoHash = "sha256-pimr4OL709EWIoLk/Wq+QAiveLyR/V70nPuzYfZSH/o=";

  nativeBuildInputs = [
    pkg-config
  ];

  buildInputs = lib.optionals stdenv.isLinux [
    udev
  ];

  meta = with lib; {
    description = "SiFli SoC serial utility command-line tool";
    homepage = "https://github.com/OpenSiFli/sftool";
    license = licenses.asl20;
    maintainers = with maintainers; [ ]; # to be filled when maintainers are added
    mainProgram = "sftool";
  };
}
