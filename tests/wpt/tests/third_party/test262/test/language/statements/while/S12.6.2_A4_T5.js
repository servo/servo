// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    "break" within a "while" Statement is allowed and performed as described
    in 12.8
es5id: 12.6.2_A4_T5
description: Using labeled "break" in order to continue a "while" loop
---*/

//CHECK#1
var i = 0;
woohoo:{
  while(true){
    i++;
    if ( i == 10 ) {
      break woohoo;
      throw new Test262Error('#1.1: "break woohoo" must break loop');
    }
  }
  throw new Test262Error('This code should be unreacheable');
}
assert.sameValue(i, 10);
