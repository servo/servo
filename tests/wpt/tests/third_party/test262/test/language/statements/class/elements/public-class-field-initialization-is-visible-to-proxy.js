// Copyright (C) 2019 Caio Lima. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Public class field initialization calls [[DefineOwnProperty]] and can be observed by Proxies
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
---*/

function ProxyBase() {
  return new Proxy(this, {
    defineProperty: function (target, key, descriptor) {
      throw new Test262Error();
    }
  });
}

class Base extends ProxyBase {
  f = "Test262";
}

assert.throws(Test262Error, () => { new Base(); });
