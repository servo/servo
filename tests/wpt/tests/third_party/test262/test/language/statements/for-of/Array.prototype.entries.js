// Copyright (C) 2013 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: >
    The method should return a valid iterator that can be traversed using a
    `for...of` loop.
es6id: 22.1.3.4
---*/

var array = [0, 'a', true, false, null, /* hole */, undefined, NaN];
var i = 0;

for (var value of array.entries()) {
  assert.sameValue(
    value[0], i, 'element at index ' + i + ': value (array key)'
  );
  assert.sameValue(
    value[1], array[i], 'element at index ' + i + ': value (array value)'
  );
  assert.sameValue(
    value.length, 2, 'element at index ' + i + ': value (array length)'
  );
  i++;
}

assert.sameValue(i, 8, 'Visits all elements');
