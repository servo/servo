// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-makesuperpropertyreference
description: >
    class super in static methods
---*/
class B {
  static method() {
    return 1;
  }
  static get x() {
    return 2;
  }
}
class C extends B {
  static method() {
    assert.sameValue(super.x, 2, "The value of `super.x` is `2`");
    return super.method();
  }
}
assert.sameValue(C.method(), 1, "`C.method()` returns `1`");
