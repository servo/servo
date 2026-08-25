// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class methods 2
---*/
class C {
  eval() {
    return 1;
  }
  arguments() {
    return 2;
  }
  static eval() {
    return 3;
  }
  static arguments() {
    return 4;
  }
};

assert.sameValue(new C().eval(), 1, "`new C().eval()` returns `1`");
assert.sameValue(new C().arguments(), 2, "`new C().arguments()` returns `2`");
assert.sameValue(C.eval(), 3, "`C.eval()` returns `3`");
assert.sameValue(C.arguments(), 4, "`C.arguments()` returns `4`");
