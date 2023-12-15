#!/usr/bin/env bash

out_dir="$(pwd)"/release-build
version=$(git describe --tags --abbrev=0)

set -x

[ -d "$out_dir" ] && rm -rf "$out_dir"
mkdir "$out_dir"

echo "$version" >"$out_dir"/VERSION_INFO
git describe --tags >"$out_dir"/BUILD_INFO
cp ./LICENSE "$out_dir"

./clean-output.py
./build-documentation.py with-docker
cp ./documentation-build/README.* "$out_dir"

cd ./AKL.Cli
dotnet publish --configuration Release --os win
cp bin/Release/net7.0/win-x64/publish/* "$out_dir"
cd -

cd "$out_dir"
zip -r -9 "another-keyboard-layer-v$version.zip" ./*
