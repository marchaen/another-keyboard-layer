#!/usr/bin/env sh

rm -rf ./akl-core-system-lib/target
rm -rf ./akl-native-lib-prototype/target

# Loop over all directories that start with the prefix "AKL." and delete their
# "bin" and "obj" folders.
for dir in "./AKL."*/
do
    rm -rf "$dir"bin
    rm -rf "$dir"obj
    rm -rf "$dir"TestResults
done
