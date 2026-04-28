// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "[[Call]] executes code associated with the object"
es5id: 8.6.2_A5_T4
description: >
    Call function-property of global object, property defined  as
    this['beep']=function(){__count++}
---*/

var __count=0;

this["beep"]=function(){__count++};
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
beep();
if (__count !==1) {
  throw new Test262Error('#1: __count=0; this["beep"]=function(){__count++}; beep(); __count === 1. Actual: ' + (__count));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#2
this["beep"]();
if (__count !==2) {
  throw new Test262Error('#2: __count=0; this["beep"]=function(){__count++}; beep(); this["beep"](); __count === 2. Actual: ' + (__count));
}
//
//////////////////////////////////////////////////////////////////////////////
