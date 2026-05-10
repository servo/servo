// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.toLowerCase()
es5id: 15.5.4.16_A1_T5
description: Call toLowerCase() function for function call
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
//since ToString(null) evaluates to "null" match(null) evaluates to match("null")
if (function() {
    return "GnulLuNa"
  }().toLowerCase() !== "gnulluna") {
  throw new Test262Error('#1: function(){return "GnulLuNa"}().toLowerCase() === "gnulluna". Actual: ' + function() {
    return "GnulLuNa"
  }().toLowerCase());
}
//
//////////////////////////////////////////////////////////////////////////////
