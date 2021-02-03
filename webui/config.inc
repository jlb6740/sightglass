#!/bin/sh

# TODO: PORTS are currently hardcoded in the docker image
# Any change here must be consistent with the docker image
SG_VIEW_PORT=3000
SG_HISTORY_PORT=8001
SG_HOSTNAME=localhost

# History file location is relative to HOST_BASE_PREFIX
SG_HISTORY_FILE="history.json"
