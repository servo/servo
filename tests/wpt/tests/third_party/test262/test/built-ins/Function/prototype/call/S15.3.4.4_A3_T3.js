// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is null or undefined, the called function is passed the global
    object as the this value
es5id: 15.3.4.4_A3_T3
description: Argument at call function is void 0
---*/

Function("this.field=\"battle\"").call(void 0);

assert.sameValue(this["field"], "battle", 'The value of this["field"] is expected to be "battle"');
