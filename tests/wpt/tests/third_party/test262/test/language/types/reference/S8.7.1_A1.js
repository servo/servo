// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    Delete operator deletes pure property, so it returns true to be applyed
    to this.property
es5id: 8.7.1_A1
description: Try to delete this.y, where y is this.y=1
---*/

this.y = 1;
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if((delete this.y) !== true){
  throw new Test262Error('#1: this.y = 1; (delete this.y) === true. Actual: ' + ((delete this.y)));
};
//
//////////////////////////////////////////////////////////////////////////////


//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (this.y !== undefined){
  throw new Test262Error('#2: this.y = 1; (delete this.y) === true; this.y === undefined. Actual: ' + (this.y));
}
//
//////////////////////////////////////////////////////////////////////////////
