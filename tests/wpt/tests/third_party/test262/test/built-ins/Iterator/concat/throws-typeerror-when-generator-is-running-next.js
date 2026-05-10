// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Throws a TypeError when the closure generator is already running.
info: |
  27.1.2.1.1 %IteratorHelperPrototype%.next ( )
    1. Return ? GeneratorResume(this value, undefined, "Iterator Helper").

  27.5.3.3 GeneratorResume ( generator, value, generatorBrand )
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
    enterCount++;
    iterator.next();
    return {done: false};
  }
}

let iterable = {
  [Symbol.iterator]() {
    return testIterator;
  }
};

let iterator = Iterator.concat(iterable);

assert.sameValue(enterCount, 0);

assert.throws(TypeError, function() {
  iterator.next();
});

assert.sameValue(enterCount, 1);
