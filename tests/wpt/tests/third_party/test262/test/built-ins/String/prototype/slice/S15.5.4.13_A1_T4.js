// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.slice (start, end)
es5id: 15.5.4.13_A1_T4
description: >
    Arguments are null and number, and instance is function call, that
    returned string
---*/

//since ToInteger(null) yelds 0
//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if (function() {
    return "gnulluna"
  }().slice(null, -3) !== "gnull") {
  throw new Test262Error('#1: function(){return "gnulluna"}().slice(null, -3) === "gnull". Actual: ' + function() {
    return "gnulluna"
  }().slice(null, -3));
}
//
//////////////////////////////////////////////////////////////////////////////
