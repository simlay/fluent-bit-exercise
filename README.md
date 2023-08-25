# Overview

This is an exercise in listening on `127.0.0.1:4242` for some simple json messages from `fluent-bit`.

# Running
```
fluent-bit -i random -o tcp://127.0.0.1:4242 -p format=json_lines &

cargo run
```


# Testing

`./tests.sh` is a non comprehensive test script. This project is about counting
even and odd values from the random data from `fluent-bit` and then sending a
`POST` request to `paste.c-net.org`
