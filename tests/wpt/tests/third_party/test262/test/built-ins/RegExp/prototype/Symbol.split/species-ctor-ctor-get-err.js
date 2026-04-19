// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 21.2.5.11
description: Behavior when error thrown while accessing `constructor` property
info: |
    [...]
    5. Let C be SpeciesConstructor(rx, %RegExp%).
    6. ReturnIfAbrupt(C).

    ES6 Section 7.3.20 SpeciesConstructor ( O, defaultConstructor )

    1. Assert: Type(O) is Object.
    2. Let C be Get(O, "constructor").
    3. ReturnIfAbrupt(C).
features: [Symbol.split]
---*/

var poisonedCtor = {
  get constructor() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, function() {
  RegExp.prototype[Symbol.split].call(poisonedCtor);
});
