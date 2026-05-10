// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Public class field initialization fails on frozen object
esid: sec-define-field
info: |
  DefineField(receiver, fieldRecord)
    ...
    8. If fieldName is a Private Name,
      a. Perform ? PrivateFieldAdd(fieldName, receiver, initValue).
    9. Else,
      a. Assert: IsPropertyKey(fieldName) is true.
      b. Perform ? CreateDataPropertyOrThrow(receiver, fieldName, initValue).
    10. Return.
features: [class, class-fields-public]
flags: [onlyStrict]
---*/

class Test {
  f = Object.freeze(this);
  g = "Test262";
}

assert.throws(TypeError, function() {
  new Test();
}, "Frozen objects can't be changed");
