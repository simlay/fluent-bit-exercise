[![codecov](https://codecov.io/gh/simlay/fluent-bit-exercise/graph/badge.svg?token=BJM75ILQKJ)](https://codecov.io/gh/simlay/fluent-bit-exercise)
# Overview

This is an exercise in listening on `127.0.0.1:4242` for some simple json messages from `fluent-bit`.

# Running
```
fluent-bit -i random -o tcp://127.0.0.1:4242 -p format=json_lines &

cargo run
```


There are also options for:
* `--addr` - the bind address to listen on. Defaults to `127.0.0.1:4242`
* `--sleep-timeout` - The amount of seconds to wait between web requests. Defaults to `60` seconds.
* `--out-file` - Where to write url response from the post requests. Defaults to `/dev/stdout`.
* `--max-count` - The number of web requests this process should do before quitting. Defaults to `u64::MAX`.


# Testing

`./tests.sh` is a non comprehensive test script. This project is about counting
even and odd values from the random data from `fluent-bit` and then sending a
`POST` request to `paste.c-net.org`
