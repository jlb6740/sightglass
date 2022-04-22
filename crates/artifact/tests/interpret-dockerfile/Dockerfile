# Some commands will be ignored, like:
FROM not-a-real-image:latest
# Check that we can change directories within our container directory.
WORKDIR /a
# Check that we can COPY a file to a directory.
COPY first.txt .
# Check that we can COPY a directory to a directory.
COPY dir/ dir/
# Check that we can run commands and even use absolute paths (as long as they are literals in this
# script).
RUN cat first.txt dir/second.txt /a/dir/third.txt | xargs > sequence.txt
# Check that we can change directories and execute commands there.
WORKDIR /b
RUN cp ../a/sequence.txt /sequence.txt

# TODO use ENV
