// Copyright (C) 2014 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
es6id: 14.5
description: >
    class name binding expression
---*/
var Cc;
var Cm;
var Cgx;
var Csx;
var Cv = class C {
  constructor() {
    assert.sameValue(C, Cv, "The value of `C` is `Cv`, inside `constructor()`");
    Cc = C;
  }
  m() {
    assert.sameValue(C, Cv, "The value of `C` is `Cv`, inside `m()`");
    Cm = C;
  }
  get x() {
    assert.sameValue(C, Cv, "The value of `C` is `Cv`, inside `get x()`");
    Cgx = C;
  }
  set x(_) {
    assert.sameValue(C, Cv, "The value of `C` is `Cv`, inside `set x()`");
    Csx = C;
  }
};

new Cv();
assert.sameValue(Cc, Cv, "The value of `Cc` is `Cv`, after executing `new Cv();`");

new Cv().m();
assert.sameValue(Cm, Cv, "The value of `Cm` is `Cv`, after executing `new Cv().m();`");

new Cv().x;
assert.sameValue(Cgx, Cv, "The value of `Cgx` is `Cv`, after executing `new Cv().x;`");

new Cv().x = 1;
assert.sameValue(Csx, Cv, "The value of `Csx` is `Cv`, after executing `new Cv().x = 1;`");
