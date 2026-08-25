// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.2-3-c-3-s
description: >
    Calling code in strict mode - eval cannot instantiate variable in
    the global context
flags: [onlyStrict]
---*/

var _10_4_2_3_c_3_s = 0;
function testcase() {
  eval("var _10_4_2_3_c_3_s = 1");
  assert.sameValue(_10_4_2_3_c_3_s, 0);
 }
testcase();
