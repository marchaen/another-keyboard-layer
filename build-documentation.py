#!/usr/bin/env python3

import shutil
import subprocess
from pathlib import Path

def generate_general_documentation(out_dir: Path):
    asciidoctor = shutil.which("asciidoctor")
    asciidoctor_pdf = shutil.which("asciidoctor-pdf")

    if asciidoctor is None or asciidoctor_pdf is None:
        print("""
    Please install asciidoctor with the asciidoctor-diagram extension and
    asciidoctor-pdf:

    https://docs.asciidoctor.org/asciidoctor/latest/install/ruby-packaging/
    https://docs.asciidoctor.org/pdf-converter/latest/install/#install-asciidoctor-pdf
    https://docs.asciidoctor.org/diagram-extension/latest/installation/
    https://docs.asciidoctor.org/asciidoctor/latest/syntax-highlighting/rouge/
              """)
        exit()

    git_commit = subprocess.run(
        "git rev-parse --short HEAD", shell=True, capture_output=True
    ).stdout.decode()

    # Html output
    subprocess.run([
        asciidoctor, "-r", "asciidoctor-diagram", "-a", "commit-hash=" + git_commit, 
        "--destination-dir", out_dir, "README.adoc"
    ])

    # Pdf output
    subprocess.run([
        asciidoctor_pdf, "-r", "asciidoctor-diagram", "-a", "commit-hash=" + git_commit,
        "--destination-dir", out_dir, "README.adoc"
    ])

    # Manpage for the cli
    subprocess.run([
        asciidoctor, "-b", "manpage", "-a", "commit-hash=" + git_commit, 
        "--destination-dir", out_dir.joinpath("man", "man1"), "Manpage.adoc"
    ])

def generate_core_system_lib_documentation():
    subprocess.run(
        "cargo doc --no-deps --document-private-items --target x86_64-pc-windows-gnu", 
        shell=True, stderr=subprocess.DEVNULL, cwd="akl-core-system-lib"
    )

    subprocess.run(
        "cargo doc --no-deps --document-private-items --target x86_64-unknown-linux-gnu", 
        shell=True, stderr=subprocess.DEVNULL, cwd="akl-core-system-lib"
    )

out_dir = Path("./documentation-build")

if out_dir.exists():
    shutil.rmtree(out_dir, True)

out_dir.mkdir()

generate_general_documentation(out_dir)
generate_core_system_lib_documentation()
