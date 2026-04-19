// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
class A {
    "constructor"() { return {}; }
}
assert.sameValue(new A() instanceof A, false);

class B extends class { } {
    "constructor"() { return {}; }
}
assert.sameValue(new B() instanceof B, false);

