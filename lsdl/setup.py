#!/usr/bin/env python3
import os
import sys
import pathlib

from setuptools import setup
from setuptools.command.test import test as TestCommand
from setuptools.command.sdist import sdist as SdistCommand

setup(
    name="lsdl",
    version= "0.0.1",
    classifiers=[
        "Intended Audience :: Developers",
        "Operating System :: POSIX",
        "Operating System :: MacOS :: MacOS X",
    ],
    packages=["lsdl"],
    include_package_data=True,
    zip_safe=False,
    cmdclass={"sdist": SdistCommand},
)
