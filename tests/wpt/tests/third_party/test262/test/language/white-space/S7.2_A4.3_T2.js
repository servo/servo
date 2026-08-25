// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Multi line comment can contain FORM FEED (U+000C)
es5id: 7.2_A4.3_T2
description: Use real FORM FEED
---*/

/*CHECK#1*/
var x = 0;
/*multilinecommentx = 1;*/
if (x !== 0) {
  throw new Test262Error('#1: var x = 0; /*multilinecommentx = 1;*/ x === 0. Actual: ' + (x));
}
