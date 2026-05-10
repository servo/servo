// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class name binding
---*/
var C2;
class C {
  constructor() {
    C2 = C;
  }
  m() {
    C2 = C;
  }
  get x() {
    C2 = C;
  }
  set x(_) {
    C2 = C;
  }
}
new C();
assert.sameValue(C, C2, "The value of `C` is `C2`");

C2 = undefined;
new C().m();
assert.sameValue(C, C2, "The value of `C` is `C2`");

C2 = undefined;
new C().x;
assert.sameValue(C, C2, "The value of `C` is `C2`");

C2 = undefined;
new C().x = 1;
assert.sameValue(C, C2, "The value of `C` is `C2`");
