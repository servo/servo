// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If thisArg is null or undefined, the called function is passed the global
    object as the this value
es5id: 15.3.4.3_A3_T7
description: >
    Argument at apply function is void 0 and it called inside function
    declaration
---*/

(function FACTORY() {
  Function("this.feat=\"in da haus\"").apply(void 0);
})();


assert.sameValue(this["feat"], "in da haus", 'The value of this["feat"] is expected to be "in da haus"');
