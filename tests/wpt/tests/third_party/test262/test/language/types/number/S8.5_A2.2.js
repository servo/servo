// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Number type represented as the extended precision 64-bit format IEEE 754
es5id: 8.5_A2.2
description: Use 2^53 + 2 number and do some operation with it
---*/

var x = 9007199254740994.0; /* 2^53 + 2 */
var y = 1.0 - 1/65536.0;
var z = x + y;
var d = z - x;

if (d === 2){
  throw new Test262Error('#1: var x = 9007199254740994.0; var y = 1.0 - 1/65536.0; var z = x + y; var d = z - x; d !== 2');
}
