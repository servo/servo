// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-makesuperpropertyreference
description: >
    class super in methods
---*/
class B {
  method() {
    return 1;
  }
  get x() {
    return 2;
  }
}
class C extends B {
  method() {
    assert.sameValue(super.x, 2, "The value of `super.x` is `2`");
    return super.method();
  }
}
assert.sameValue(new C().method(), 1, "`new C().method()` returns `1`");
