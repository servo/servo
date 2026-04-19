// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class getters 2
---*/
class C {
  get eval() {
    return 1;
  }
  get arguments() {
    return 2;
  }
  static get eval() {
    return 3;
  }
  static get arguments() {
    return 4;
  }
};

assert.sameValue(new C().eval, 1, "The value of `new C().eval` is `1`");
assert.sameValue(new C().arguments, 2, "The value of `new C().arguments` is `2`");
assert.sameValue(C.eval, 3, "The value of `C.eval` is `3`");
assert.sameValue(C.arguments, 4, "The value of `C.arguments` is `4`");
