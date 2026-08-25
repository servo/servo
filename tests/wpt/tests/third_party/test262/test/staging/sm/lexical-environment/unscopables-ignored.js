// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
// In these cases, @@unscopables should not be consulted.

// Because obj has no properties `assert.sameValue` or `x`,
// obj[@@unscopables] is not checked here:
var obj = {
    get [Symbol.unscopables]() {
        throw "tried to read @@unscopables";
    }
};
var x = 3;
with (obj)
    assert.sameValue(x, 3);

// If @@unscopables is present but not an object, it is ignored:
for (let nonObject of [undefined, null, "nothing", Symbol.for("moon")]) {
    let y = 4;
    let obj2 = {[Symbol.unscopables]: nonObject, y: 5};
    with (obj2)
        assert.sameValue(y, 5);
}

