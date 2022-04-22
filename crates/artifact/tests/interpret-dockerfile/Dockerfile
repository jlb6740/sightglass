# Some commands will be ignored, like:
FROM not-a-real-image:latest
# Check that we can change directories within our container directory.
WORKDIR /a
# Check that we can COPY a file to a directory.
COPY first.txt .
# Check that we can COPY a directory to a directory.
COPY dir/ dir/
# We can set build arguments with defaults that are overridable from outside.
ARG INPUT=first.txt
ARG OUTPUT=some/nonsense/path/overriden/in/the/test
# Check that we can run commands and even use absolute paths (as long as they are literals in this
# script).
RUN cat $INPUT dir/second.txt /a/dir/third.txt | xargs > $OUTPUT
# We can also set build arguments
# Check that we can change directories and execute commands there.
WORKDIR /b
RUN cp ../a/$OUTPUT /sequence.txt
