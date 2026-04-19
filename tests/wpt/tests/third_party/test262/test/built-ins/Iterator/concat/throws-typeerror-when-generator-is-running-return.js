// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Throws a TypeError when the closure generator is already running.
info: |
  27.1.2.1.2 %IteratorHelperPrototype%.return ( )
    ...
    6. Return ? GeneratorResumeAbrupt(O, C, "Iterator Helper").

  27.5.3.4 GeneratorResumeAbrupt ( generator, abruptCompletion, generatorBrand )
    1. Let state be ? GeneratorValidate(generator, generatorBrand).
    ...

  27.5.3.2 GeneratorValidate ( generator, generatorBrand )
    ...
    6. If state is executing, throw a TypeError exception.
    ...
features: [iterator-sequencing]
---*/

let enterCount = 0;

let testIterator = {
  next() {
    return {done: false};
  },
  return() {
    enterCount++;
    iterator.return();
    return {done: false};
  }
}

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator = Iterator.concat(iterable);

iterator.next();

assert.sameValue(enterCount, 0);

assert.throws(TypeError, function() {
  iterator.return();
});

assert.sameValue(enterCount, 1);
