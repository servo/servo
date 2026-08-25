// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: PrivateFieldSet should return an abrupt completion
esid: sec-privatefieldset
info: |
  PrivateFieldSet (P, O, value)
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
  set #m(_) {
    throw new Test262Error();
  }

  access() {
    this.#m = 'Test262';
  }
}

let c = new C();
assert.throws(Test262Error, function() {
  c.access();
}, 'private setter should have abrupt completion');
