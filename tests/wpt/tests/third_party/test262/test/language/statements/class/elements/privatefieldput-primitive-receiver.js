// Copyright (C) 2020 Caio Lima (Igalia S.L). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: PrivateField calls ToObject when receiver is a primitive
esid: sec-putvalue
info: |
  PutValue ( V, W )
    ...
    6. If IsPropertyReference(V), then
      a. If HasPrimitiveBase(V), then
        i. Assert: In this case, base will never be null or undefined.
        ii. Let base be ToObject(base).
      b. If IsPrivateReference(V), then
        i. Return ? PrivateFieldSet(field, base, W).
    ...

  PrivateFieldSet (P, O, value )
    1. Assert: P is a Private Name.
    2. Assert: Type(O) is Object.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is empty, throw a TypeError exception.
    5. Set entry.[[PrivateFieldValue]] to value.

features: [class, class-fields-private, BigInt]
---*/

let count = 0;

class C {
  #p = 1;

  method(v) {
    count++;
    try {
      count++;
      this.#p = v;
    } catch (e) {
      count++;
      if (e instanceof TypeError) {
        throw new Test262Error();
      }
    }
  }
}

assert.throws(Test262Error, () => {
  new C().method.call(15, 0);
});
assert.sameValue(count, 3);

assert.throws(Test262Error, () => {
  new C().method.call('Test262', 0);
});
assert.sameValue(count, 6);

assert.throws(Test262Error, () => {
  new C().method.call(Symbol('Test262'), 0);
});
assert.sameValue(count, 9);

assert.throws(Test262Error, () => {
  new C().method.call(15n, 0);
});
assert.sameValue(count, 12);

assert.throws(Test262Error, () => {
  new C().method.call(null, 0);
});
assert.sameValue(count, 15);

assert.throws(Test262Error, () => {
  new C().method.call(undefined, 0);
});
assert.sameValue(count, 18);

