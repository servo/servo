// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Every function call enters a new execution context
es5id: 10.4A1.1_T2
description: Recursive function call
---*/

var y;

function f(a){
  var x;

  if (a === 1)
    return x;
  else {
    if(x === undefined) {
      x = 0;
    } else {
      x = 1;
    }
    return f(1);
  }
}

y = f(0);

if(!(y === undefined)){
  throw new Test262Error("#1: Recursive function calls shares execution context");
}
