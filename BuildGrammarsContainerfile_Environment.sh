#!/bin/bash

export TERM=xterm-kitty

podman build -f BuildGrammarsContainerfile -t ltlsp-dev -v $(pwd):/workspace
podman run -it \
    --userns=keep-id \
    --mount type=bind,source=$(pwd),destination=/workspace,chown=false \
    --env "TERM=kitty" \
     ltlsp-dev bash
