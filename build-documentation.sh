#!/usr/bin/env sh

out_dir="./documentation-build"

# Clean old output
if [ -d $out_dir ]; then
    rm -rf $out_dir
fi

mkdir $out_dir

# Check that asciidoctor is installed
if ! [ -x "$(command -v asciidoctor)" ]; then
  echo 'Please install asciidoctor with the asciidoctor-diagram extension.
https://docs.asciidoctor.org/asciidoctor/latest/install/ruby-packaging/
https://docs.asciidoctor.org/diagram-extension/latest/installation/
https://docs.asciidoctor.org/asciidoctor/latest/syntax-highlighting/rouge/' >&2
  exit 1
fi

# Generate documentation
asciidoctor -r asciidoctor-diagram -acommit-hash=$(git rev-parse --short HEAD) --destination-dir $out_dir README.adoc
