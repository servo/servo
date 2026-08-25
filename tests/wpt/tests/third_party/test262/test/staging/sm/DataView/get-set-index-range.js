// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

var buffer = new ArrayBuffer(2);
var view = new DataView(buffer);

function check(view) {
    for (let fun of ['getInt8', 'setInt8', 'getInt16', 'setInt16']) {
        assert.throws(RangeError, () => view[fun](-10));
        assert.throws(RangeError, () => view[fun](-Infinity));
        assert.throws(RangeError, () => view[fun](Infinity));

        assert.throws(RangeError, () => view[fun](Math.pow(2, 53)));
        assert.throws(RangeError, () => view[fun](Math.pow(2, 54)));
    }
}

check(view);

for (let fun of ['getInt8', 'getInt16']) {
    assert.sameValue(view[fun](0), 0);
    assert.sameValue(view[fun](undefined), 0);
    assert.sameValue(view[fun](NaN), 0);
}

// ToIndex is called before detachment check, so we can tell the difference
// between a ToIndex failure and a real out of bounds failure.
$DETACHBUFFER(buffer);

check(view);

assert.throws(TypeError, () => view.getInt8(0));
assert.throws(TypeError, () => view.setInt8(0, 0));
assert.throws(TypeError, () => view.getInt8(Math.pow(2, 53) - 1));
assert.throws(TypeError, () => view.setInt8(Math.pow(2, 53) - 1, 0));
