# Benchmarks for Open Web Platform Storage.

These benchmarks exercise storage apis in a real-life usage way (avoiding microbenchmarks).

# IDB Docs Load

This models an offline load of a Google doc. See [this document](https://docs.google.com/document/d/1JC1RgMyxBAjUPSHjm2Bd1KPzcqpPPvxRomKevOkMPm0/edit) for a breakdown of the database and the transactions, along with the traces used to extract this information.

# Blob Perf

This benchmark models the creation and reading of blobs. It has two parts:

 1. Create and read blobs synchronously
 2. Create and read blobs in parallel (asynchronously).

There is a variant of this test for every transportation type (shared memory, files, and ipc).