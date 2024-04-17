#!/bin/bash

podman build -f BuildGrammarsContainerfile -t ltlsp-dev -v $(pwd):/workspace
podman run -i -v $(pwd):/workspace ltlsp-dev:latest bash
