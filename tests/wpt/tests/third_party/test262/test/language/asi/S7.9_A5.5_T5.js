// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Check Function Expression for automatic semicolon insertion
es5id: 7.9_A5.5_T5
description: Insert some LineTerminators into rerutn expression;
---*/

//CHECK#1
var x =
1 + (function f
(t){
return {
a:
function(){
return t + 1
}
}
}
)
(2 + 3).
a
()

if (x !== 7) {
  throw new Test262Error('#1: Check Function Expression for automatic semicolon insertion');
}
