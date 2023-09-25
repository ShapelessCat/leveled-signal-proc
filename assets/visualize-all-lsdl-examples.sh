#!/usr/bin/zsh

set -ex
REPO_DIRECTORY=$(dirname $(readlink -f $0))/..
cargo build --release --bin lsp-ir-dot-graph

pushd ${REPO_DIRECTORY}/lsdl/examples
make clean && make
popd

for FILE in ${REPO_DIRECTORY}/lsdl/examples/*.json; do
    EXAMPLE_NAME=$(basename ${FILE} .json)
    ${REPO_DIRECTORY}/target/release/lsp-ir-dot-graph ${FILE} | dot -Tsvg -o ${REPO_DIRECTORY}/assets/lsdl-example-svg/${EXAMPLE_NAME}.svg
done
