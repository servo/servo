// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.search (regexp)
es5id: 15.5.4.12_A1_T8
description: >
    Argument is void 0, and instance is String object with overrided
    toString function
---*/

var __obj = {
  toString: function() {}
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(void 0) evaluates to "undefined" search(void 0) evaluates to search("undefined")
if (String(__obj).search(void 0) !== 0) {
  throw new Test262Error('#1: __obj = {toString:function(){}}; String(__obj).search(void 0) === 0. Actual: ' + String(__obj).search(void 0));
}
//
//////////////////////////////////////////////////////////////////////////////
