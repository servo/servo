// Copyright 2015 Microsoft Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
description: >
    Source is an object with length property and one item is deleted
    from the source
esid: sec-array.from
es6id: 22.1.2.1
---*/

var array = [2, 4, 0, 16];
var expectedArray = [2, 4, , 16];
var obj = {
  length: 4,
  0: 2,
  1: 4,
  2: 0,
  3: 16
};
delete obj[2];
var a = Array.from(obj);
for (var j = 0; j < expectedArray.length; j++) {
  assert.sameValue(a[j], expectedArray[j], 'The value of a[j] is expected to equal the value of expectedArray[j]');
}
