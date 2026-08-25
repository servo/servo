// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is null or undefined, the called function is passed the global
    object as the this value
es5id: 15.3.4.3_A3_T9
description: Checking by using eval, argument at apply function is void 0
---*/

eval(" Function(\"this.feat=1\").apply(void 0) ");

assert.sameValue(this["feat"], 1, 'The value of this["feat"] is expected to be 1');
