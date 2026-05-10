// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  pending
esid: pending
---*/
// Test that yield* can appear in a loop, and alongside yield.

function* countdown(n) {
    while (n > 0) {
        yield n;
        yield* countdown(--n);
    }
    return 34;
}

function collect_results(iter) {
    var ret = [];
    var result;
    do {
        result = iter.next();
        ret.push(result);
    } while (!result.done);
    return ret;
}

var expected = [
    // yield in countdown(3), n == 3
    {value: 3, done: false},
    // yield in yield* countdown(2), n == 2
    {value: 2, done: false},
    // yield in nested yield* countdown(1), n == 1
    {value: 1, done: false},
    // countdown(0) yields no values
    // second go-through of countdown(2) loop, n == 1
    {value: 1, done: false},
    // second go-through of countdown(3) loop, n == 2
    {value: 2, done: false},
    // yield in yield* countdown(1), n == 1
    {value: 1, done: false},
    // third go-through of countdown(3) loop, n == 1
    {value: 1, done: false},
    // done
    {value: 34, done: true}
];

assert.deepEqual(expected, collect_results(countdown(3)));

