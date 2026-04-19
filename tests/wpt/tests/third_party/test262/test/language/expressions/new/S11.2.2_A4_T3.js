// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If NewExpression or MemberExpression does not implement internal
    [[Construct]] method, throw TypeError
es5id: 11.2.2_A4_T3
description: Checking String object case
---*/

//CHECK#1
try {
  new new String("1");
  throw new Test262Error('#1: new new String("1") throw TypeError');	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#1: new new String("1") throw TypeError');	
  }
}

//CHECK#2
try {
  var x = new String("1");
  new x;
  throw new Test262Error('#2: var x = new String("1"); new x throw TypeError');	
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#2: var x = new String("1"); new x throw TypeError');	
  }
}

//CHECK#3
try {
  var x = new String("1");
  new x();
  throw new Test262Error('#3: var x = new String("1"); new x() throw TypeError');  
}
catch (e) {
  if ((e instanceof TypeError) !== true) {
    throw new Test262Error('#3: var x = new String("1"); new x() throw TypeError');  
  }
}
