// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class setters 2
---*/
var x = 0;
class C {
  set eval(v) {
    x = v;
  }
  set arguments(v) {
    x = v;
  }
  static set eval(v) {
    x = v;
  }
  static set arguments(v) {
    x = v;
  }
};

new C().eval = 1;
assert.sameValue(x, 1, "The value of `x` is `1`");
new C().arguments = 2;
assert.sameValue(x, 2, "The value of `x` is `2`");
C.eval = 3;
assert.sameValue(x, 3, "The value of `x` is `3`");
C.arguments = 4;
assert.sameValue(x, 4, "The value of `x` is `4`");
