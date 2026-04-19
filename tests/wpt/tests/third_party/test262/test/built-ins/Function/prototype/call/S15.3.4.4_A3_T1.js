// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is null or undefined, the called function is passed the global
    object as the this value
es5id: 15.3.4.4_A3_T1
description: Not any arguments at call function
---*/

Function("this.field=\"strawberry\"").call();

assert.sameValue(this["field"], "strawberry", 'The value of this["field"] is expected to be "strawberry"');
