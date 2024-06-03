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
    git fetch --all --prune --tags --depth 1
  else
    pushd "$FOLDER"
    git fetch --all --prune --tags --depth 1
  fi

  git checkout "$REF"

  if [ ! -f "Makefile" ]; then
    tree-sitter generate grammar.js
  fi
  
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
# build_tree_sitter "tree-sitter-markdown" "https://github.com/tree-sitter-grammars/tree-sitter-markdown.git" "v0.2.3" "libtree-sitter-markdown.so"
build_tree_sitter "tree-sitter-git-commit" "https://github.com/the-mikedavis/tree-sitter-git-commit.git" "6f193a66e9aa872760823dff020960c6cedc37b3" "libtree-sitter-git_commit.so"

popd
popd
