// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: Source is an object with missing values
esid: sec-array.from
es6id: 22.1.2.1
---*/

var array = [2, 4, , 16];
var obj = {
  length: 4,
  0: 2,
  1: 4,
  3: 16
};

var a = Array.from.call(Object, obj);
assert.sameValue(typeof a, "object", 'The value of `typeof a` is expected to be "object"');
for (var j = 0; j < a.length; j++) {
  assert.sameValue(a[j], array[j], 'The value of a[j] is expected to equal the value of array[j]');
}
