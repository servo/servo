// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  The value of the [[Prototype]] internal slot of the return value of Iterator.prototype.flatMap is the
  intrinsic object %IteratorHelperPrototype%.
features: [iterator-helpers]
---*/

assert(
  (function* () {})().flatMap(() => []) instanceof Iterator,
  'function*(){}().flatMap(() => []) must return an Iterator'
);
