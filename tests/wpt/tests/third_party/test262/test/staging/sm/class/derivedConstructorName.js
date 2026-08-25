// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class A {
    constructor() { }
}

class B extends A { }

var b = new B();
assert.sameValue(b.constructor.name, "B");

