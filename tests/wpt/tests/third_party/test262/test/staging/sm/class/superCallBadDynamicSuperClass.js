// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class base { constructor() { } }

class inst extends base { constructor() { super(); } }
Object.setPrototypeOf(inst, Math.sin);
assert.throws(TypeError, () => new inst());

class defaultInst extends base { }
Object.setPrototypeOf(inst, Math.sin);
assert.throws(TypeError, () => new inst());

