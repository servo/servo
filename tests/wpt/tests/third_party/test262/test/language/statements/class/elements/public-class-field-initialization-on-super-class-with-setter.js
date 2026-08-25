// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Public class field initialization calls [[DefineOwnProperty]] and don't execute super's getter
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
includes: [propertyHelper.js]
features: [class, class-fields-public]
---*/

class Super {
  set f(v) {
    throw new Test262Error();
  }
}

class Base extends Super {
  f = "Test262";
}

let o = new Base();

verifyProperty(o, "f", {
  value: "Test262",
  enumerable: true,
  writable: true,
  configurable: true,
});
