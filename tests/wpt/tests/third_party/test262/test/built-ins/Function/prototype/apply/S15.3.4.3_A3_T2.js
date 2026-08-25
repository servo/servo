// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is null or undefined, the called function is passed the global
    object as the this value
es5id: 15.3.4.3_A3_T2
description: Argument at apply function is null
---*/

Function("this.field=\"green\"").apply(null);

assert.sameValue(this["field"], "green", 'The value of this["field"] is expected to be "green"');
