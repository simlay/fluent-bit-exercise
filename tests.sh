#!/bin/bash

TIMEOUT=10
MAX_REQUESTS=5
URL_FILE=./urls.txt
ADDR="127.0.0.1:4242"

# We do this because cargo llvm-cov doesn't have a built step
cargo llvm-cov run -- --sleep-timeout 1 --max-count 0 > /dev/null

cargo llvm-cov run --lcov --output-path lcov.info -- --sleep-timeout $TIMEOUT --max-count $MAX_REQUESTS --out-file $URL_FILE &

# We hope that this is long enough for rust to start listening.
sleep 6



for j in {1..2}; do
    ## EVEN SENDS
    for i in {1..10}; do
        echo "{\"date\": 0.0, \"rand_value\": $(($i*2))}"; sleep 0.1;
    done | socat - tcp:$ADDR
    sleep $TIMEOUT

    # ODD SENDS
    for i in {1..10}; do
        echo "{\"date\": 0.0, \"rand_value\": $(($i*2 + 1))}"; sleep 0.1;
    done | socat - tcp:$ADDR
    sleep $TIMEOUT
done

sleep $TIMEOUT

cat $URL_FILE
OUT=$(for i in $(cat ${URL_FILE}); do curl -s $i; done)
echo $OUT
EXPECTED="odd=0 even=10odd=10 even=0odd=0 even=10odd=10 even=0"

if [ "${OUT}" != "${EXPECTED}" ]; then
    echo OUT PUT DOES NOT MATCH
    exit 1
fi
