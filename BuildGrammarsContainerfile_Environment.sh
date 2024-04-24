#!/bin/bash

podman build -f BuildGrammarsContainerfile -t ltlsp-dev -v $(pwd):/workspace
podman run -i \
    --userns=keep-id \
    --mount type=bind,source=$(pwd),destination=/workspace,chown=false \
    --env "TERM=$TERM" \
     ltlsp-dev bash
