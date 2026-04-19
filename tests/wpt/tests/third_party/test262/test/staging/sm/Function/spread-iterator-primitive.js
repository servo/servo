// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  Returning non-object from @@iterator should throw
info: bugzilla.mozilla.org/show_bug.cgi?id=1021835
esid: pending
---*/

let primitives = [
    1,
    true,
    undefined,
    null,
    "foo",
    Symbol.iterator
];

function f() {
}

for (let primitive of primitives) {
    let arg = {
        [Symbol.iterator]() {
            return primitive;
        }
    };
    assert.throws(TypeError, () => f(...arg));
}
