#!/bin/bash

# Check if directory exists
#if [ ! -d "sg-view/node_modules" ]; then
#    echo "Kill servers"
#    killall node -q
#    killall sg-history -q
#fi
#echo "VIEW PORT: $2"

# Check if port alive
check_view_return=$(lsof -i -P -n | grep LISTEN | grep ^node | grep :${1})
if [ -z "$check_view_return" ]; then

    # start a development version of the app
    echo "Generating Files"
    node_modules/.bin/nuxt generate
    cd ../
fi
#echo "Done with view"
