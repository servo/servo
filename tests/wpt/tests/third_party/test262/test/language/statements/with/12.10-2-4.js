// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: ToObject conversion from undefined value must throw TypeError
es5id: 12.10-2-4
description: Trying to convert undefined to Object
flags: [noStrict]
---*/

try{
  with(undefined) x = 2;
  throw new Test262Error('#2.1: with(undefined) x = 2 must throw TypeError. Actual: x === ' + (x));
}
catch(e){
  if((e instanceof TypeError) !== true){
    throw new Test262Error('#2.2: with(undefined) x = 2 must throw TypeError. Actual: ' + (e));
  }
}
