.DEFAULT_GOAL := reinstall

.DELETE_ON_ERROR:

WHEEL=$(shell ls ./dist/*.whl)

clean:
	rm -rf build lsdl.egg-info dist

build:
	python -m build --sdist --wheel

uninstall:
	pip uninstall lsdl

install:
	pip install "${WHEEL}"

reinstall: clean build uninstall install
