// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.drop
description: >
  The value of the [[Prototype]] internal slot of the return value of Iterator.prototype.drop is the
  intrinsic object %IteratorHelperPrototype%.
features: [iterator-helpers]
---*/

assert((function* () {})().drop(0) instanceof Iterator, 'function*(){}().drop(0) must return an Iterator');
