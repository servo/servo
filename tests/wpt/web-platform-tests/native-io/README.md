This directory contains tests for the
[NativeIO API](https://github.com/TODO/native-io).

## Note on the synchronous APIs

Chrome is currently working with developers to explore the performance
implications of using NativeIO as an asynchronous Promise-based API from
WebAssembly.

In order to assess the performance overhead, a baseline is needed. This baseline
is a synchronous API that can be easily used to port existing database code to
WebAssembly. The synchronous API is only exposed to dedicated workers.

Until our performance studies are concluded, Chrome has no plans of shipping the
synchronous API. In other words, there are no plans of adding a new synchronous
storage API to the Web Platform.
