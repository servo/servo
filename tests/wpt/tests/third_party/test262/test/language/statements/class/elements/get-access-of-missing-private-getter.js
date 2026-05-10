// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Trying to get a private member without getter throws TypeError
esid: sec-privatefieldget
info: |
  PrivateFieldGet ( P, O )
    1. Assert: P is a Private Name.
    2. If O is not an object, throw a TypeError exception.
    3. If P.[[Kind]] is "field",
      a. Let entry be PrivateFieldFind(P, O).
      b. If entry is empty, throw a TypeError exception.
      c. Return entry.[[PrivateFieldValue]].
    4. Perform ? PrivateBrandCheck(O, P).
    5. If P.[[Kind]] is "method",
      a. Return P.[[Value]].
    6. Else,
      a. Assert: P.[[Kind]] is "accessor".
      b. If P does not have a [[Get]] field, throw a TypeError exception.
      c. Let getter be P.[[Get]].
      d. Return ? Call(getter, O).
features: [class-methods-private, class]
---*/

class C {
  set #f(v) {
    throw new Test262Error();
  }

  getAccess() {
    return this.#f;
  }
}

let c = new C();
assert.throws(TypeError, function() {
  c.getAccess();
}, 'get operation on private accessor without getter should throw TypeError');
