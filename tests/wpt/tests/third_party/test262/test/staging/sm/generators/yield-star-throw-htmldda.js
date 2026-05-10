// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
features:
  - IsHTMLDDA
description: |
  pending
esid: pending
---*/
function* g(iter) {
    yield* iter;
}

var calledReturn = false;

var it = g({
    [Symbol.iterator]() {
        return this;
    },
    next() {
        return {done: false};
    },
    throw: $262.IsHTMLDDA,
    return() {
        calledReturn = true;
        return {done: false};
    }
});

it.next();

assert.throws(TypeError, () => it.throw(""));

assert.sameValue(calledReturn, false);

