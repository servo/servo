// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object-environment-records-setmutablebinding-n-v-s
description: >
  Compound Assignment Operator calls PutValue(lref, v) (formerly S11.13.2_A5.9_T4)
info: |
  The concrete Environment Record method SetMutableBinding for object Environment
  Records attempts to set the value of the Environment Record's associated binding
  object's property whose name is the value of the argument N to the value of argument V.
  A property named N normally already exists but if it does not or is not currently writable,
  error handling is determined by the value of the Boolean argument S.

  Let stillExists be ? HasProperty(bindings, N).
  If stillExists is false and S is true, throw a ReferenceError exception.
flags: [noStrict]
---*/
var count = 0;
var scope = {
  get x() {
    delete this.x;
    return 5;
  }
};

with (scope) {
  (function() {
    "use strict";
    assert.throws(ReferenceError, () => {
      count++;
      x &= 3;
      count++;
    });
    count++;
  })();
}

assert.sameValue(count, 2);
assert(!('x' in scope));
