#!/bin/bash

TIMEOUT=10
MAX_REQUESTS=5
URL_FILE=./urls.txt
ADDR="127.0.0.1:4242"

# We do this because cargo llvm-cov doesn't have a built step
cargo llvm-cov run -- --sleep-timeout 1 --max-count 0 > /dev/null

cargo llvm-cov run -- --sleep-timeout $TIMEOUT --max-count $MAX_REQUESTS --out-file $URL_FILE &

# We hope that this is long enough for rust to start listening.
sleep 3



for j in 0..2; do
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

cat $URL_FILE
OUT=$(for i in $(cat ${URL_FILE}); do curl $i; done)
echo $OUT
