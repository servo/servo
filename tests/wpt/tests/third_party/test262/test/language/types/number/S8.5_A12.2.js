// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: -Infinity is the same as Number.NEGATIVE_INFINITY
es5id: 8.5_A12.2
description: Compare -Infinity with Number.NEGATIVE_INFINITY
---*/

var n_inf=-Infinity;

//CHECK #1
if (n_inf !== Number.NEGATIVE_INFINITY){
  throw new Test262Error('#1: -Infinity is the same as Number.NEGATIVE_INFINITY');
}
