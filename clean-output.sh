#!/usr/bin/env sh

rm -rf ./akl-core-system-lib/target

# Loop over all directories that start with the prefix "AKL." and delete their
# "bin" and "obj" folders.
for dir in "./AKL."*/
do
    rm -rf "$dir"bin
    rm -rf "$dir"obj
done
