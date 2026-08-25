// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 10.4.2-3-c-2-s
description: >
    Calling code in strict mode - eval cannot instantiate variable in
    the variable environment of the calling context
flags: [onlyStrict]
---*/

function testcase() {
  var _10_4_2_3_c_2_s = 0;
  function _10_4_2_3_c_2_sFunc() {
     eval("var _10_4_2_3_c_2_s = 1");
     assert.sameValue(_10_4_2_3_c_2_s, 0);
  }
  _10_4_2_3_c_2_sFunc();
 }
testcase();
