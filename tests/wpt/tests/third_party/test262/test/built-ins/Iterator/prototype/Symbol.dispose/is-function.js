// Copyright (C) 2023 Ron Buckton. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-%iteratorprototype%-@@dispose
description: >
  Iterator.prototype[@@dispose] is a built-in function
features: [explicit-resource-management]
---*/
const IteratorPrototype = Object.getPrototypeOf(
  Object.getPrototypeOf([][Symbol.iterator]())
);

assert.sameValue(typeof IteratorPrototype[Symbol.dispose], 'function');
