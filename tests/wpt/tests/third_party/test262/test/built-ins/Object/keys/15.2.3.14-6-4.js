// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-6-4
description: >
    Object.keys - the order of elements in returned array is the same
    with the order of properties in 'O' (Arguments object)
---*/

var func = function(a, b, c) {
  return arguments;
};

var args = func(1, "b", false);

var tempArray = [];
for (var p in args) {
  if (args.hasOwnProperty(p)) {
    tempArray.push(p);
  }
}

var returnedArray = Object.keys(args);

for (var index in returnedArray) {
  assert.sameValue(tempArray[index], returnedArray[index], 'tempArray[index]');
}
