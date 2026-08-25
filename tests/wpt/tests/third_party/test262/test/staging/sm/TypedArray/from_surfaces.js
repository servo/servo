// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
includes: [sm/non262-TypedArray-shell.js]
description: |
  pending
esid: pending
---*/
for (var constructor of anyTypedArrayConstructors) {
    // Check superficial features of %TypeArray%.from.
    var desc = Object.getOwnPropertyDescriptor(constructor.__proto__, "from");
    assert.sameValue(desc.configurable, true);
    assert.sameValue(desc.enumerable, false);
    assert.sameValue(desc.writable, true);
    assert.sameValue(constructor.from.length, 1);
    assert.throws(TypeError, () => new constructor.from());  // not a constructor
}

