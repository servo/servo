// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-makesuperpropertyreference
description: >
    class super in getter
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
  get y() {
    assert.sameValue(super.x, 2, "The value of `super.x` is `2`");
    return super.method();
  }
}
assert.sameValue(new C().y, 1, "The value of `new C().y` is `1`");
