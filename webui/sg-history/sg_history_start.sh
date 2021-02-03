#!/bin/bash

. "$(dirname ${BASH_SOURCE:-$0})/../config.inc"
: "${SG_HISTORY_IMAGE_NAME:=sg_history_image}"
: "${SG_HISTORY_CONTAINER_NAME:=sg_history_container}"

HOST_BASE_PREFIX="${HOST_BASE_PREFIX:-$(cd "$(dirname ${BASH_SOURCE:-$0})" && pwd)}/"

if ! docker image inspect ${SG_HISTORY_IMAGE_NAME} > /dev/null; then
	echo  "${SG_HISTORY_IMAGE_NAME} image does not exist ... exiting"
    exit
fi

if [[ $(docker ps --all -f "name=${SG_HISTORY_CONTAINER_NAME}" --format '{{.Names}}') == ${SG_HISTORY_CONTAINER_NAME} ]]; then
    if [ $(docker inspect -f '{{.State.Running}}' ${SG_HISTORY_CONTAINER_NAME}) == 'false' ]; then
        echo "Starting ${SG_HISTORY_CONTAINER_NAME}" >&2
        docker start ${SG_HISTORY_CONTAINER_NAME}
    else
        echo "${SG_HISTORY_CONTAINER_NAME} is already running. Restarting" >&2
        docker restart ${SG_HISTORY_CONTAINER_NAME}
    fi
else
    if [[ ! -e ${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE} ]]; then
        cp ${HOST_BASE_PREFIX}/../history.json.bak ${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE}
    fi

    docker run -it --name=${SG_HISTORY_CONTAINER_NAME} --detach --volume  ${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE} \
    -p ${SG_HISTORY_PORT}:${SG_HISTORY_PORT} \
    --mount type=bind,source=${HOST_BASE_PREFIX}/sg_history_launch.sh,target=/sg-history/sg_history_launch.sh \
    --mount type=bind,source=${HOST_BASE_PREFIX}/../${SG_HISTORY_FILE},target=/sg-history/history.json \
    ${SG_HISTORY_IMAGE_NAME} tail -f /dev/null
fi
echo "SG_HISTORY_PORT: ${SG_HISTORY_PORT}"
docker exec ${SG_HISTORY_CONTAINER_NAME} sh -c "/sg-history/sg_history_launch.sh ${SG_HISTORY_PORT}" &
