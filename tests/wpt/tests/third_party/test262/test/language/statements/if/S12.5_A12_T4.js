// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Embedded "if/else" constructions are allowed
es5id: 12.5_A12_T4
description: Using embedded "if" into "if" constructions
---*/

//CHECK# 1
if(true)
  if (false)
    throw new Test262Error('#1.1: At embedded "if/else" constructions engine must select right branches');

//CHECK# 2
var c=0;
if(true)
  if (true)
    c=2;
if (c!==2)
    throw new Test262Error('#2: At embedded "if/else" constructions engine must select right branches');

//CHECK# 3
if(false)
  if (true)
    throw new Test262Error('#3.1: At embedded "if/else" constructions engine must select right branches');

//CHECK# 4
if(false)
  if (true)
    throw new Test262Error('#4.1: At embedded "if/else" constructions engine must select right branches');
