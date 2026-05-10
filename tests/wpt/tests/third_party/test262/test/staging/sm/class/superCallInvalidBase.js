// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class instance extends null {
    constructor() { super(); }
}

assert.throws(TypeError, () => new instance());
assert.throws(TypeError, () => new class extends null { }());

