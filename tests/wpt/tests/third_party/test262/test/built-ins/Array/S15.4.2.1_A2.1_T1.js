// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The length property of the newly constructed object;
    is set to the number of arguments
es5id: 15.4.2.1_A2.1_T1
description: Array constructor is given no arguments or at least two arguments
---*/
assert.sameValue(new Array().length, 0, 'The value of new Array().length is expected to be 0');
assert.sameValue(new Array(0, 1, 0, 1).length, 4, 'The value of new Array(0, 1, 0, 1).length is expected to be 4');

assert.sameValue(
  new Array(undefined, undefined).length,
  2,
  'The value of new Array(undefined, undefined).length is expected to be 2'
);
