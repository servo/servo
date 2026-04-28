// Copyright (C) 2023 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.take
description: >
  Throws a TypeError when the closure generator is already running.
info: |
  %IteratorHelperPrototype%.next ( )
    1. Return ? GeneratorResume(this value, undefined, "Iterator Helper").

  27.5.3.3 GeneratorResume ( generator, value, generatorBrand )
    1. Let state be ? GeneratorValidate(generator, generatorBrand).
    ...

  27.5.3.2 GeneratorValidate ( generator, generatorBrand )
    ...
    6. If state is executing, throw a TypeError exception.
    ...

features: [iterator-helpers]
---*/

var enterCount = 0;

class TestIterator extends Iterator {
  next() {
    enterCount++;
    iter.next();
    return {done: false};
  }
}

var iter = new TestIterator().take(100);

assert.throws(TypeError, function() {
  iter.next();
});

assert.sameValue(enterCount, 1);
