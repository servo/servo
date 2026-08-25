// Copyright (C) 2016 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-runtime-semantics-classdefinitionevaluation
description: >
  Default class constructor does not use argument evaluation.
features: [Symbol.iterator]
---*/

Array.prototype[Symbol.iterator] = function() {
  throw new Test262Error('@@iterator invoked');
};

class Base {
  constructor(value) {
    this.value = value;
  }
}

class Derived extends Base {}

const instance = new Derived(5);

assert.sameValue(instance.value, 5);
