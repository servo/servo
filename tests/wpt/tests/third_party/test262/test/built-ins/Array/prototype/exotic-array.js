// Copyright 2009 the Sputnik authors.  All rights reserved.
// Copyright (C) 2017 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-properties-of-the-array-prototype-object
description: >
  Array.prototype is an exoctic array object
info: |
  22.1.3 Properties of the Array Prototype Object

  (...)
  The Array prototype object is an Array exotic object and has the internal
  methods specified for such objects.
---*/

Array.prototype[2] = 42;

assert.sameValue(Array.prototype.length, 3);
assert.sameValue(Array.prototype[0], undefined, 'Array.prototype[0]');
assert.sameValue(Array.prototype[1], undefined, 'Array.prototype[1]');
assert.sameValue(Array.prototype[2], 42, 'Array.prototype[2]');

assert.sameValue({}.toString.call(Array.prototype), '[object Array]');
