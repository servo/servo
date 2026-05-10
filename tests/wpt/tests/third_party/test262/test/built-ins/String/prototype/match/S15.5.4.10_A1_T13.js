// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.match (regexp)
es5id: 15.5.4.10_A1_T13
description: >
    Override toString and valueOf functions, then call match (regexp)
    function with this object as argument
---*/

var __obj = {
  toString: function() {
    return {};
  },
  valueOf: function() {
    return 1;
  }
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ("ABB\u0041B\u0031ABAB\u0031BBAA".match(__obj)[0] !== "1") {
  throw new Test262Error('#1: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; "ABB\\u0041B\\u0031ABAB\\u0031BBAA".match(__obj)[0] ==="1". Actual: ' + ("ABB\u0041B\u0031ABAB\u0031BBAA".match(__obj)[0]));
}
//
//////////////////////////////////////////////////////////////////////////////

//////////////////////////////////////////////////////////////////////////////
//CHECK#3
if ("ABB\u0041B\u0031ABAB\u0031BBAA".match(__obj).length !== 1) {
  throw new Test262Error('#3: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; "ABB\\u0041B\\u0031ABAB\\u0031BBAA".match(__obj).length === 1. Actual: ' + ("ABB\u0041B\u0031ABAB\u0031BBAA".match(__obj).length));
}
//
//////////////////////////////////////////////////////////////////////////////
