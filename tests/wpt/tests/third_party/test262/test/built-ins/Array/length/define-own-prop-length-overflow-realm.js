// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-array-exotic-objects-defineownproperty-p-desc
es6id: 9.4.2.1
description: >
  Error when setting a length larger than 2**32 (honoring the Realm of the
  current execution context)
info: |
  [...]
  2. If P is "length", then
     a. Return ? ArraySetLength(A, Desc).
features: [cross-realm]
---*/

var OArray = $262.createRealm().global.Array;
var array = new OArray();

assert.throws(RangeError, function() {
  array.length = 4294967296;
}, 'array.length = 4294967296 throws a RangeError exception');
