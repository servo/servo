// Copyright (C) 2024 André Bargull and Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  The value of the [[Prototype]] internal slot of the return value of Iterator.concat
  is the intrinsic object %IteratorHelperPrototype%.
features: [iterator-sequencing]
---*/

var iter = Iterator.concat();
assert(iter instanceof Iterator, "Iterator.concat() must return an Iterator");

var customIter = { next() { return { done: true, value: undefined }; } };
iter = Iterator.concat({ [Symbol.iterator]() { return customIter; } });
assert(iter instanceof Iterator, "Iterator.concat(...) must return an Iterator");
