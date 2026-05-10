// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    [[Get]](P) method should return undefined when property P does not exist
    both in instance and prototype
es5id: 8.12.3_A2
description: >
    Try to get P when property P does not exist both in instance and
    prototype
---*/

var __obj={};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (__obj.propFoo !== undefined){
  throw new Test262Error('#1: var __obj={}; __obj.propFoo === undefined. Actual: ' + (__obj.propFoo));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
if (__obj['propFoo'] !== undefined){
  throw new Test262Error('#2: var __obj={}; __obj[\'propFoo\'] === undefined. Actual: ' + (__obj['propFoo']));
}
//
//////////////////////////////////////////////////////////////////////////////
