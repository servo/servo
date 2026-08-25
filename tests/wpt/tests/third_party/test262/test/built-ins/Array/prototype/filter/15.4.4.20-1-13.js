// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-array.prototype.filter
description: Array.prototype.filter applied to the JSON object
---*/

function callbackfn(val, idx, obj) {
  return '[object JSON]' === Object.prototype.toString.call(JSON);
}

JSON.length = 1;
JSON[0] = 1;
var newArr = Array.prototype.filter.call(JSON, callbackfn);

assert.sameValue(newArr[0], 1, 'newArr[0]');
