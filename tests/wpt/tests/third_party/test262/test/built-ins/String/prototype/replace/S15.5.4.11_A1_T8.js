// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.replace (searchValue, replaceValue)
es5id: 15.5.4.11_A1_T8
description: >
    Call replace (searchValue, replaceValue) function with regular
    expression and void 0 arguments of String object
---*/

var __obj = {
  toString: function() {}
};

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (String(__obj).replace(/e/g, void 0) !== "undundefinedfinundefinedd") {
  throw new Test262Error('#1: __obj = {toString:function(){}}; String(__obj).replace(/e/g,void 0) === "undundefinedfinundefinedd". Actual: ' + String(__obj).replace(/e/g, void 0));
}
//
//////////////////////////////////////////////////////////////////////////////
