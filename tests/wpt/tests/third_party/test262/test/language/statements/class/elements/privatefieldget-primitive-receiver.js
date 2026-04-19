// Copyright (C) 2020 Caio Lima (Igalia S.L). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: PrivateField calls ToObject when receiver is a primitive
esid: sec-getvalue
info: |
  GetValue ( V )
    ...
    5. If IsPropertyReference(V), then
      a. If HasPrimitiveBase(V), then
        i. Assert: In this case, base will never be null or undefined.
        ii. Let base be ToObject(base).
      b. If IsPrivateReference(V), then
        i. Return ? PrivateFieldGet(field, base).
    ...

  PrivateFieldGet (P, O )
    1. Assert: P is a Private Name value.
    2. If O is not an object, throw a TypeError exception.
    3. Let entry be PrivateFieldFind(P, O).
    4. If entry is empty, throw a TypeError exception.
    5. Return entry.[[PrivateFieldValue]].

features: [class, class-fields-private, BigInt]
---*/

let count = 0;

class C {
  #p = 1;

  method() {
    count++;
    try {
      count++;
      this.#p;
    } catch (e) {
      count++;
      if (e instanceof TypeError) {
        throw new Test262Error();
      }
    }
  }
}

assert.throws(Test262Error, () => {
  new C().method.call(15);
});
assert.sameValue(count, 3);

assert.throws(Test262Error, () => {
  new C().method.call('Test262');
});
assert.sameValue(count, 6);

assert.throws(Test262Error, () => {
  new C().method.call(Symbol('Test262'));
});
assert.sameValue(count, 9);

assert.throws(Test262Error, () => {
  new C().method.call(15n);
});
assert.sameValue(count, 12);

assert.throws(Test262Error, () => {
  new C().method.call(null);
});
assert.sameValue(count, 15);

assert.throws(Test262Error, () => {
  new C().method.call(undefined);
});
assert.sameValue(count, 18);

