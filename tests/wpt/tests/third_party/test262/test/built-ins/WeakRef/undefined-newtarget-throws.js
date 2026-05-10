// Copyright (C) 2019 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-weak-ref-target
description: >
  Throws a TypeError if NewTarget is undefined.
info: |
  WeakRef ( target )

  1. If NewTarget is undefined, throw a TypeError exception.
  2. If Type(target) is not Object, throw a TypeError exception.
  ...
features: [WeakRef]
---*/

assert.sameValue(
  typeof WeakRef, 'function',
  'typeof WeakRef is function'
);

assert.throws(TypeError, function() {
  WeakRef();
});

assert.throws(TypeError, function() {
  WeakRef({});
});
