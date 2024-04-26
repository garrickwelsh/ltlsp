#!/bin/bash

function build_tree_sitter() {
  FOLDER="$1"
  GRAMMAR="$2"
  REF="$3"
  LIBRARY="$4"

  # curl "$GRAMMAR" -o grammar.js
  # tree-sitter generate

  if [ ! -d "$FOLDER" ]; then
    git clone --depth 1 https://github.com/tree-sitter/tree-sitter-rust.git

    mkdir -p "$FOLDER"
    pushd "$FOLDER"
  else
    pushd "$FOLDER"
    git fetch --all --prune
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

popd
popd
