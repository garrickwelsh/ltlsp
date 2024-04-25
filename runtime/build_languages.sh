#!/bin/bash

function build_tree_sitter() {
  FOLDER="$1"
  GRAMMAR="$2"
  LIBRARY="$3"

  mkdir -p "$FOLDER"
  pushd "$FOLDER"

  curl "$GRAMMAR" -o grammar.js
  tree-sitter generate
  make
  #cp "$LIBRARY" 
  popd
}

pushd "$(dirname "$0")"

build_tree_sitter "rust" "https://raw.githubusercontent.com/tree-sitter/tree-sitter-rust/master/grammar.js" "libtree-sitter-rust.so"

popd
