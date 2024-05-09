#!/bin/sh

HERE=`dirname $0`
cd "${HERE}"

if [ ! -d sppark_bal ]; then
    trap '[ -h sppark_bal ] && rm -f sppark_bal' 0 2
    ln -s .. sppark_bal
fi

# --allow-dirty because the temporary sppark symbolic link is not committed
cargo +stable publish --allow-dirty "$@"
