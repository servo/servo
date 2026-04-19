// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js, detachArrayBuffer.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    var receiver = {};

    var ta = new constructor(1);
    assert.sameValue(Reflect.set(ta, 0, 47, receiver), true);
    assert.sameValue(ta[0], 0);
    assert.sameValue(receiver[0], 47);

    // Out-of-bounds
    assert.sameValue(Reflect.set(ta, 10, 47, receiver), true);
    assert.sameValue(ta[10], undefined);
    assert.sameValue(receiver[10], undefined);
    assert.sameValue(Object.hasOwn(receiver, 10), false);

    // Detached
    if (!isSharedConstructor(constructor)) {
        $DETACHBUFFER(ta.buffer)

        assert.sameValue(ta[0], undefined);
        assert.sameValue(Reflect.set(ta, 0, 42, receiver), true);
        assert.sameValue(ta[0], undefined);
        assert.sameValue(receiver[0], 47);
    }
}

