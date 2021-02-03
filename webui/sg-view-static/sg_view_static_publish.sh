#!/bin/bash

. "$(dirname ${BASH_SOURCE:-$0})/../config.inc"
: "${SG_VIEW_STATIC_IMAGE_NAME:=sg_view_static_image}"
: "${SG_VIEW_STATIC_CONTAINER_NAME:=sg_view_static_container}"

HOST_BASE_PREFIX="${HOST_BASE_PREFIX:-$(cd "$(dirname ${BASH_SOURCE:-$0})" && pwd)}/"
echo "Here 1"
if ! docker image inspect ${SG_VIEW_STATIC_IMAGE_NAME} > /dev/null; then
    echo  "${SG_VIEW_STATIC_IMAGE_NAME} image does not exist ... exiting"
    exit
fi
echo "Here 2"
if [[ $(docker ps --all -f "name=${SG_VIEW_STATIC_CONTAINER_NAME}" --format '{{.Names}}') == ${SG_VIEW_STATIC_CONTAINER_NAME} ]]; then
    if [ $(docker inspect -f '{{.State.Running}}' ${SG_VIEW_STATIC_CONTAINER_NAME}) == 'false' ]; then
        echo "Starting ${SG_VIEW_STATIC_CONTAINER_NAME}" >&2
        docker start ${SG_VIEW_STATIC_CONTAINER_NAME}
    else
        echo "${SG_VIEW_STATIC_CONTAINER_NAME} is already running. Restarting" >&2
        docker restart ${SG_VIEW_STATIC_CONTAINER_NAME}

    fi
else
    if [[ ! -e ${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE} ]]; then
        echo "No history file found .. exiting"
        exit
    fi
echo "Here 3"
    docker run -it --name=${SG_VIEW_STATIC_CONTAINER_NAME} --detach \
    --volume ${HOST_BASE_PREFIX}/dist/:/sg-static-view/dist/  \
    --mount type=bind,source=${HOST_BASE_PREFIX}/sg_view_static_launch.sh,target=/sg-static-view/sg_view_static_launch.sh \
    --mount type=bind,source=${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE},target=/sg-static-view/public/history.json \
    ${SG_VIEW_STATIC_IMAGE_NAME} tail -f /dev/null
fi
echo "Here 4"
echo "Launch Webui"
cp ${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE} ${HOST_BASE_PREFIX}/public/${SG_HISTORY_FILE}
docker exec ${SG_VIEW_STATIC_CONTAINER_NAME} sh -c "/sg-static-view/sg_view_static_launch.sh"

#download and push
date=$(date '+%Y-%m-%d')
echo "Push \"Update Aarch64 ${date}\""

if [ ! -d github_webserver ]; then
    git clone https://github.com/jlb6740/jlb6740.github.io.git github_webserver
fi
cd github_webserver
git clean -fd
git reset --hard
git pull
rsync -avh  ${HOST_BASE_PREFIX}/../../../webui_runner/results Aarch64_results --delete
cd Aarch64/
cp ${HOST_BASE_PREFIX}/public/${SG_HISTORY_FILE} ${HOST_BASE_PREFIX}/dist/
rsync -avh ${HOST_BASE_PREFIX}/dist/ . --delete
git add -A
date=$(date '+%Y-%m-%d')
git commit -m "Update Aarch64 ${date}"
git push origin HEAD
cd ../../
