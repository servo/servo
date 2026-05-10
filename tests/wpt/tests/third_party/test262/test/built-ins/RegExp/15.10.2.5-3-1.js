// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.10.2.5-3-1
description: >
    Term - SyntaxError was thrown when max is finite and less than min
    (15.10.2.5 step 3)
---*/

assert.throws(SyntaxError, function() {
  var regExp = new RegExp("0{2,1}");
});
