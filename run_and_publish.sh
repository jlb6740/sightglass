HOST_BASE_PREFIX="${HOST_BASE_PREFIX:-$(cd "$(dirname ${BASH_SOURCE:-$0})" && pwd)}/"
cd ${HOST_BASE_PREFIX}

# Build Containers
cd webui_runner ; ./sg_image_build.sh ; cd ..
cd webui/sg-history ; ./sg_history_build.sh ; cd ../../
cd webui/sg-view-static/ ; ./sg_view_static_build.sh ; cd ../../

# Initiate Database Server (History)
cd webui/sg-history ; ./sg_history_start.sh ; cd ../../

# Run
cd webui_runner/ ; ./sg_container_runner.sh -r wasmtime_app -p shootout -s ; cd ../

# Publish
cd webui/sg-view-static/ ; ./sg_view_static_publish.sh ; cd ../../