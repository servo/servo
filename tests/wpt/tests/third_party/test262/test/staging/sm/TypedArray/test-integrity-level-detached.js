// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
const EMPTY = 0;
const INLINE_STORAGE = 10;
const NON_INLINE_STORAGE = 1024;

class DetachedInt32Array extends Int32Array {
    constructor(...args) {
        super(...args);
        $DETACHBUFFER(this.buffer);
    }
}

// Empty typed arrays can be sealed.
{
    let ta = new DetachedInt32Array(EMPTY);
    Object.seal(ta);

    assert.sameValue(Object.isExtensible(ta), false);
    assert.sameValue(Object.isSealed(ta), true);
    assert.sameValue(Object.isFrozen(ta), true);
}

// Non-empty typed arrays can be sealed, but calling TestIntegrityLevel will
// throw on detached typed arrays.
for (let length of [INLINE_STORAGE, NON_INLINE_STORAGE]) {
    let ta = new DetachedInt32Array(length);
    Object.seal(ta);

    assert.sameValue(Object.isExtensible(ta), false);
    assert.sameValue(Object.isSealed(ta), true);
    assert.sameValue(Object.isFrozen(ta), true);
}

// Empty typed arrays can be frozen.
{
    let ta = new DetachedInt32Array(EMPTY);
    Object.freeze(ta);

    assert.sameValue(Object.isExtensible(ta), false);
    assert.sameValue(Object.isSealed(ta), true);
    assert.sameValue(Object.isFrozen(ta), true);
}

// Non-empty typed arrays cannot be frozen.
for (let length of [INLINE_STORAGE, NON_INLINE_STORAGE]) {
    let ta = new DetachedInt32Array(length);
    Object.freeze(ta);

    assert.sameValue(Object.isExtensible(ta), false);
    assert.sameValue(Object.isSealed(ta), true);
    assert.sameValue(Object.isFrozen(ta), true);
}

// Non-extensible empty typed arrays are sealed and frozen.
{
    let ta = new DetachedInt32Array(EMPTY);
    Object.preventExtensions(ta);

    assert.sameValue(Object.isExtensible(ta), false);
    assert.sameValue(Object.isSealed(ta), true);
    assert.sameValue(Object.isFrozen(ta), true);
}

// Calling TestIntegrityLevel will throw on detached typed arrays with non-zero
// length.
for (let length of [INLINE_STORAGE, NON_INLINE_STORAGE]) {
    let ta = new DetachedInt32Array(length);
    Object.preventExtensions(ta);

    assert.sameValue(Object.isExtensible(ta), false);
    assert.sameValue(Object.isSealed(ta), true);
    assert.sameValue(Object.isFrozen(ta), true);
}


