#!/usr/bin/env python3

import shutil
from pathlib import Path

build_directory_patterns = [
    "./*/bin",
    "./*/obj",
    "./*/TestResults",
    "./akl-*/target",
    "./akl-*/build-*",
]

for pattern in build_directory_patterns:
    for directory in Path(".").glob(pattern):
        shutil.rmtree(directory, ignore_errors=True)
