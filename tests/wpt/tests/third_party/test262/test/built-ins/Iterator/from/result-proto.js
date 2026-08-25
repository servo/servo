// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  The value of the [[Prototype]] internal slot of the return value of Iterator.from is the
  intrinsic object %WrapForValidIteratorPrototype%, whose [[Prototype]] is %IteratorHelperPrototype%.
features: [iterator-helpers]
---*/
let iter = {
  next() {
    return {
      done: true,
      value: undefined,
    };
  },
};

const WrapForValidIteratorPrototype = Object.getPrototypeOf(Iterator.from(iter));

assert.sameValue(Object.getPrototypeOf(WrapForValidIteratorPrototype), Iterator.prototype);

class SubIterator extends Iterator {}
assert.sameValue(Object.getPrototypeOf(SubIterator.from(iter)), WrapForValidIteratorPrototype);

function* g() {}
const GeneratorPrototype = Object.getPrototypeOf(g());

assert.sameValue(Object.getPrototypeOf(Iterator.from(g())), GeneratorPrototype);
