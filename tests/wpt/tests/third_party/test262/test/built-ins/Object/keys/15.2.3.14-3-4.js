// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.14-3-4
description: >
    Object.keys of an arguments object returns the indices of the
    given arguments
---*/

function testArgs2(x, y, z) {
  // Properties of the arguments object are enumerable.
  var a = Object.keys(arguments);
  if (a.length === 2 && a[0] in arguments && a[1] in arguments)
    return true;
}

function testArgs3(x, y, z) {
  // Properties of the arguments object are enumerable.
  var a = Object.keys(arguments);
  if (a.length === 3 && a[0] in arguments && a[1] in arguments && a[2] in arguments)
    return true;
}

function testArgs4(x, y, z) {
  // Properties of the arguments object are enumerable.
  var a = Object.keys(arguments);
  if (a.length === 4 && a[0] in arguments && a[1] in arguments && a[2] in arguments && a[3] in arguments)
    return true;
}

assert(testArgs2(1, 2), 'testArgs2(1, 2) !== true');
assert(testArgs3(1, 2, 3), 'testArgs3(1, 2, 3) !== true');
assert(testArgs4(1, 2, 3, 4), 'testArgs4(1, 2, 3, 4) !== true');
