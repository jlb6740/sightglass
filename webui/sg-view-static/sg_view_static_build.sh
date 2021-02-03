#!/bin/bash

. "$(dirname ${BASH_SOURCE:-$0})/../config.inc"
: "${SG_VIEW_STATIC_IMAGE_NAME:=sg_view_static_image}"

echo "Build $SG_VIEW_STATIC_IMAGE_NAME"

if docker image inspect $SG_VIEW_STATIC_IMAGE_NAME > /dev/null; then
        if [ -z "$FORCE_REBUILD" ]; then
                echo "A sg_webui image is already present"
                echo "Hit Ctrl-C right now if you don't want to rebuild it"
                echo "or skip this wait by setting FORCE_REBUILD=1"
                sleep 6
        fi
fi

docker build -t ${SG_VIEW_STATIC_IMAGE_NAME} .
