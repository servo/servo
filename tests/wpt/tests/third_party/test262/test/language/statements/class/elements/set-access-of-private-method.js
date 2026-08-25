// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Trying to set a private method throws TypeError
esid: sec-privatefieldset
info: |
  PrivateFieldSet ( P, O, value )
    1. Assert: P is a Private Name.
    2. If O is not an object, throw a TypeError exception.
    3. If P.[[Kind]] is "field",
      a. Let entry be PrivateFieldFind(P, O).
      b. If entry is empty, throw a TypeError exception.
      c. Set entry.[[PrivateFieldValue]] to value.
      d. Return.
    4. If P.[[Kind]] is "method", throw a TypeError exception.
    5. Else,
      a. Assert: P.[[Kind]] is "accessor".
      b. If O.[[PrivateFieldBrands]] does not contain P.[[Brand]], throw a TypeError exception.
      c. If P does not have a [[Set]] field, throw a TypeError exception.
      d. Let setter be P.[[Set]].
      e. Perform ? Call(setter, O, value).
      f. Return.
features: [class-methods-private, class]
---*/

class C {
  #f() {
    throw new Test262Error();
  }

  setAccess() {
    this.#f = 'Test262';
  }
}

let c = new C();
assert.throws(TypeError, function() {
  c.setAccess();
}, 'set operation on private method should throw TypeError');

