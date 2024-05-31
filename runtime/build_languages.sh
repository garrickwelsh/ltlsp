#!/bin/bash

function build_tree_sitter() {
  FOLDER="$1"
  GRAMMAR="$2"
  REF="$3"
  LIBRARY="$4"

  # curl "$GRAMMAR" -o grammar.js
  # tree-sitter generate

  if [ ! -d "$FOLDER" ]; then
    echo git clone --depth 1 "$GRAMMAR"
    git clone --depth 1 "$GRAMMAR"
    mkdir -p "$FOLDER"
    pushd "$FOLDER"
  else
    pushd "$FOLDER"
    git fetch --all --prune --tags
  fi

  git checkout "$REF"
  
  make
  cp "$LIBRARY" ../../ltlsp_grammars
  popd
}

pushd "$(dirname "$0")"

mkdir -p ltlsp_grammars
mkdir -p ltlsp_grammars_build
pushd ltlsp_grammars_build

build_tree_sitter "tree-sitter-rust" "https://github.com/tree-sitter/tree-sitter-rust" "v0.21.2" "libtree-sitter-rust.so"
build_tree_sitter "tree-sitter-c-sharp" "https://github.com/tree-sitter/tree-sitter-c-sharp" "v0.21.1" "libtree-sitter-c_sharp.so"
build_tree_sitter "tree-sitter-go" "https://github.com/tree-sitter/tree-sitter-go" "v0.21.0" "libtree-sitter-go.so"

popd
popd
