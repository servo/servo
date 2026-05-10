// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-makesuperpropertyreference
description: >
    class super in constructor
---*/
var calls = 0;
class B {}
B.prototype.x = 42;

class C extends B {
  constructor() {
    super();
    calls++;
    assert.sameValue(super.x, 42, "The value of `super.x` is `42`");
  }
}

new C;
assert.sameValue(calls, 1, "The value of `calls` is `1`");
