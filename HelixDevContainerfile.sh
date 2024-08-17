#!/bin/bash

export TERM=xterm-kitty

podman build -f HelixDevContainerfile -t ltlsp -v $(pwd):/workspace
podman run --security-opt label:disable -it \
    --userns=keep-id \
    --mount type=bind,source=$(pwd),destination=/workspace,chown=false \
    --mount type=bind,source=$HOME/.config,destination=/home/gaz/.config,chown=false,readonly=true \
    --env "TERM=kitty" \
     ltlsp bash
