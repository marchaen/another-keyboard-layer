#!/usr/bin/env python3

import sys
from pathlib import Path
import shutil
import subprocess


def main():
    out_dir = Path("./documentation-build")

    if out_dir.exists():
        shutil.rmtree(out_dir, True)

    out_dir.mkdir()

    # True if executed like this: ./build-documentation.py with-docker
    with_docker = len(sys.argv) == 2 and sys.argv[1] == "with-docker"
    generate_general_documentation(with_docker, out_dir)


def get_asciidoctor_binaries(use_docker: bool) -> (str, str):
    if use_docker:
        return ("asciidoctor", "asciidoctor-pdf")

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

    return (asciidoctor, asciidoctor_pdf)


def execute_command(description: str, command: str, use_result=False, cwd="."):
    prefix = "[{}]".format(description)
    print(prefix, command)

    result = subprocess.run(
        command,
        shell=True,
        capture_output=use_result,
        cwd=cwd,
    )

    if use_result:
        result = result.stdout.decode().strip()

        for line in result.split('\n'):
            if line and line.isspace():
                continue
            print(prefix, line)

        return result


def generate_general_documentation(use_docker: bool, out_dir: Path):
    asciidoctor, asciidoctor_pdf = get_asciidoctor_binaries(use_docker)

    git_commit = execute_command(
        "Get git commit",
        "git rev-parse --short HEAD",
        True
    )

    base_command = (
        "{binary} -r asciidoctor-diagram -a commit-hash=" + git_commit +
        " --destination-dir {out_dir} {file}"
    )

    if use_docker:
        container = execute_command(
            "Building custom asciidoc container",
            "docker build -f ./gen-docs.Dockerfile -q .",
            True
        )

        base_command = (
            "docker run --rm -u $(id -u):$(id -g) -v $(pwd):/documents/ "
            "{} {}".format(container, base_command)
        )

    # Html output
    execute_command(
        "Generating documentation in html format",
        base_command.format(
            binary=asciidoctor,
            out_dir=out_dir,
            file="README.adoc"
        )
    )

    # Pdf output
    execute_command(
        "Generating documentation in pdf format",
        base_command.format(
            binary=asciidoctor_pdf,
            out_dir=out_dir,
            file="README.adoc"
        )
    )

    # Manpage for the cli
    execute_command(
        "Generating Manpage for the cli",
        base_command.format(
            binary=asciidoctor,
            out_dir=out_dir.joinpath("man", "man1"),
            file="Manpage.adoc"
        )
    )

    if use_docker:
        # There seems to be a problem for the java installation to find the home
        # directory of the logged in user so that it can store a font config
        # cache ($HOME/.java/fonts/...) there. So a directory with the name
        # "/.java/..." (The symbol is U+F03F) is created outside the container.
        #
        # The following statement deletes a directory which has a name that is
        # exactly one non-displayable character long.
        shutil.rmtree("./?")
        return

if __name__ == "__main__":
    main()
