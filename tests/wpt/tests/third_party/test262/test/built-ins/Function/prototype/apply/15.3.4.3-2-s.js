// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.3-2-s
description: >
    Strict Mode - 'this' value is a number which cannot be converted
    to wrapper objects when the function is called with an array of
    arguments
flags: [onlyStrict]
---*/

function fun() {
  return (this instanceof Number);
}

assert.sameValue(fun.apply(-12, Array), false, 'fun.apply(-12, Array)');
