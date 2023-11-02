#!/usr/bin/env bash

out_dir=$(pwd)"/release-build"

set -x

[ -d $out_dir ] && rm -rf $out_dir

mkdir $out_dir

./clean-output.py

./build-documentation.py
cp ./documentation-build/README.* $out_dir
cp ./LICENSE $out_dir

git describe --tags --abbrev=0 > "$out_dir"/VERSION_INFO
git describe --tags > "$out_dir"/BUILD_INFO

cd ./AKL.Cli
dotnet publish --configuration Release --os win
cp bin/Release/net7.0/win-x64/publish/* $out_dir
