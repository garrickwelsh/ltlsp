#!/bin/bash

function build_tree_sitter() {
  FOLDER="$1"
  GRAMMAR="$2"
  REF="$3"
  LIBRARY="$4"

  # curl "$GRAMMAR" -o grammar.js
  # tree-sitter generate

  git clone --depth 1 https://github.com/tree-sitter/tree-sitter-rust.git

  mkdir -p "$FOLDER"
  pushd "$FOLDER"

  git checkout 473634230435c18033384bebaa6d6a17c2523281
  
  make
  #cp "$LIBRARY" 
  popd
}

pushd "$(dirname "$0")"

build_tree_sitter "tree-sitter-rust" "https://github.com/tree-sitter/tree-sitter-rust.git" "473634230435c18033384bebaa6d6a17c2523281" "libtree-sitter-rust.so"
# build_tree_sitter "rust" "https://raw.githubusercontent.com/tree-sitter/tree-sitter-rust/master/grammar.js" "libtree-sitter-rust.so"

popd
