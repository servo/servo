// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.3.4.4-3-s
description: >
    Strict Mode - 'this' value is a boolean which cannot be converted
    to wrapper objects when the function is called without an array of
    arguments
flags: [onlyStrict]
---*/

function fun() {
  return (this instanceof Boolean);
}

assert.sameValue(fun.call(false), false, 'fun.call(false)');
