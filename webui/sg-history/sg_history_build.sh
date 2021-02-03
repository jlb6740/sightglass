#!/bin/bash

. "$(dirname ${BASH_SOURCE:-$0})/../config.inc"
: "${SG_HISTORY_IMAGE_NAME:=sg_history_image}"

echo "Building $SG_HISTORY_IMAGE_NAME"

if docker image inspect ${SG_HISTORY_IMAGE_NAME} > /dev/null; then
        if [ -z "$FORCE_REBUILD" ]; then
                echo "An image is already present"
                echo "Hit Ctrl-C right now if you don't want to rebuild it"
                echo "or skip this wait by setting FORCE_REBUILD=1"
                sleep 4
        fi
fi

docker build -t ${SG_HISTORY_IMAGE_NAME} .
