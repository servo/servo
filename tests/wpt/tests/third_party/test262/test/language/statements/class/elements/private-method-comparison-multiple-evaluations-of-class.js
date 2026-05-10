// Copyright (C) 2019 Caio Lima (Igalia SL). All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Every new evaluation of a class creates a different brand (private getter)
esid: sec-privatefieldget
info: |
  PrivateFieldGet (P, O)
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

  PrivateBrandCheck(O, P)
    1. If O.[[PrivateBrands]] does not contain an entry e such that SameValue(e, P.[[Brand]]) is true,
      a. Throw a TypeError exception.
features: [class, class-methods-private]
---*/

let createAndInstantiateClass = function () {
  class C {
    #m() { return 'test262'; }
    
    getPrivateMethod() {
      return this.#m;
    }
  }
  
  return new C();
};

let c1 = createAndInstantiateClass();
let c2 = createAndInstantiateClass();

assert.notSameValue(c1.getPrivateMethod(), c2.getPrivateMethod());
