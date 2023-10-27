#!/usr/bin/env python3

import shutil
import subprocess
from pathlib import Path

out_dir = Path("./documentation-build")

if out_dir.exists():
    shutil.rmtree(out_dir, True)

out_dir.mkdir()

asciidoctor = shutil.which("asciidoctor")

if asciidoctor is None:
    print("""
Please install asciidoctor with the asciidoctor-diagram extension:

https://docs.asciidoctor.org/asciidoctor/latest/install/ruby-packaging/
https://docs.asciidoctor.org/diagram-extension/latest/installation/
https://docs.asciidoctor.org/asciidoctor/latest/syntax-highlighting/rouge/'
          """)
    exit()

git_commit = subprocess.run(
    "git rev-parse --short HEAD", shell=True, capture_output=True
).stdout.decode()

subprocess.run([
    asciidoctor, "-r", "asciidoctor-diagram", "-acommit-hash=" + git_commit, 
    "--destination-dir", out_dir, "README.adoc"
])

subprocess.run(
    "cargo doc --no-deps --document-private-items", shell=True,
    stderr=subprocess.DEVNULL, cwd="akl-core-system-lib"
)
