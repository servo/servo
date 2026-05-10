// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: ToObject conversion from null value must throw TypeError
es5id: 12.10-2-5
description: Trying to convert null to Object
flags: [noStrict]
---*/

try{
  with(null) x = 2;
  throw new Test262Error('#2.1: with(null) x = 2 must throw TypeError. Actual: x === . Actual: ' + (x));
}
catch(e){
  if((e instanceof TypeError) !== true){
    throw new Test262Error('#2.2: with(null) x = 2 must throw TypeError. Actual: ' + (e));
  }
}
