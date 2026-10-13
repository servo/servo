# Working on crown

While there are test files, running them won't show print output.
Write the test and then debug crown by running it manually on those tests:

```
RUST_BACKTRACE=1 cargo run -- \
      "-Zcrate-attr=feature(register_tool)" \
      "-Zcrate-attr=register_tool(crown)" tests/<test-file-here>
```
