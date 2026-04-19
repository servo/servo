// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp)
es5id: 15.5.4.12_A1_T13
description: >
    Argument is object, and instance is string.  Object with overrided
    toString and valueOf functions
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
if ("ABB\u0041B\u0031ABAB\u0031BBAA".search(__obj) !== 5) {
  throw new Test262Error('#1: var __obj = {toString:function(){return {};},valueOf:function(){return 1;}}; "ABB\\u0041B\\u0031ABAB\\u0031BBAA".search(__obj) ===5. Actual: ' + ("ABB\u0041B\u0031ABAB\u0031BBAA".search(__obj)));
}
//
//////////////////////////////////////////////////////////////////////////////
