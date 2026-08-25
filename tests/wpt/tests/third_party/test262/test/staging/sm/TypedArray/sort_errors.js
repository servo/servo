// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/

// Ensure that TypedArrays throw when attempting to sort a detached ArrayBuffer
assert.throws(TypeError, () => {
    let buffer = new ArrayBuffer(32);
    let array  = new Int32Array(buffer);
    $DETACHBUFFER(buffer);
    array.sort();
});

// Ensure detaching buffer in comparator doesn't throw an error.
{
    let detached = false;
    let ta = new Int32Array(3);
    ta.sort(function(a, b) {
        if (!detached) {
            detached = true;
            $DETACHBUFFER(ta.buffer);
        }
        return a - b;
    });
    assert.sameValue(detached, true);
}

let otherGlobal = $262.createRealm().global;

// Ensure detachment check doesn't choke on wrapped typed array.
{
    let ta = new Int32Array(3);
    otherGlobal.Int32Array.prototype.sort.call(ta, function(a, b) {
        return a - b;
    });
}

// Ensure detaching buffer in comparator doesn't throw an error when the typed array is wrapped.
{
    let detached = false;
    let ta = new Int32Array(3);
    otherGlobal.Int32Array.prototype.sort.call(ta, function(a,b) {
        if (!detached) {
            detached = true;
            $DETACHBUFFER(ta.buffer);
        }
        return a - b;
    });
    assert.sameValue(detached, true);
}

// Ensure that TypedArray.prototype.sort will not sort non-TypedArrays
assert.throws(TypeError, () => {
    let array = [4, 3, 2, 1];
    Int32Array.prototype.sort.call(array);
});

assert.throws(TypeError, () => {
    Int32Array.prototype.sort.call({a: 1, b: 2});
});

assert.throws(TypeError, () => {
    Int32Array.prototype.sort.call(Int32Array.prototype);
});

assert.throws(TypeError, () => {
    let buf = new ArrayBuffer(32);
    Int32Array.prototype.sort.call(buf);
});

// Ensure that comparator errors are propagataed
function badComparator(x, y) {
    if (x == 99 && y == 99)
        throw new TypeError;
    return x - y;
}

assert.throws(TypeError, () => {
    let array = new Uint8Array([99, 99, 99, 99]);
    array.sort(badComparator);
});

assert.throws(TypeError, () => {
    let array = new Uint8Array([1, 99, 2, 99]);
    array.sort(badComparator);
});


