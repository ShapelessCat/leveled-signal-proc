#!/usr/bin/env python3
from setuptools import setup
from setuptools.command.sdist import sdist as sdist_command

setup(
    name="lsdl",
    version="0.0.1",
    classifiers=[
        "Intended Audience :: Developers",
        "Operating System :: POSIX",
        "Operating System :: MacOS :: MacOS X",
        "Operating System :: Windows",
    ],
    packages=["lsdl", "lsdl.signal_processors", "lsdl.measurements"],
    include_package_data=True,
    zip_safe=False,
    cmdclass={"sdist": sdist_command},
)
