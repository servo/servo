// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [deepEqual.js]
description: |
  String.prototype.match should call GetMethod.
info: bugzilla.mozilla.org/show_bug.cgi?id=1290655
esid: pending
---*/

function create(value) {
    return {
        [Symbol.match]: value,
        toString() {
            return "-";
        }
    };
}

var expected = ["-"];
expected.index = 1;
expected.input = "a-a";
expected.groups = undefined;

for (let v of [null, undefined]) {
    assert.deepEqual("a-a".match(create(v)), expected);
}

for (let v of [1, true, Symbol.iterator, "", {}, []]) {
    assert.throws(TypeError, () => "a-a".match(create(v)));
}
