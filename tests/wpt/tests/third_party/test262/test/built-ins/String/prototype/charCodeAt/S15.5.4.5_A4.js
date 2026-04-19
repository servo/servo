// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    When String.prototype.charCodeAt(pos) calls first calls ToString, giving
    it the this value as its argument
es5id: 15.5.4.5_A4
description: Change toString function, it trow exception, and call charCodeAt()
---*/

var __obj = {
  valueOf: 1,
  toString: function() {
    throw 'intostring'
  },
  charCodeAt: String.prototype.charCodeAt
}

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
try {
  var x = __obj.charCodeAt();
  throw new Test262Error('#1:  "var x = __obj.charCodeAt()" lead to throwing exception');
} catch (e) {
  if (e !== 'intostring') {
    throw new Test262Error('#1.1: Exception === \'intostring\'. Actual: exception ===' + e);
  }
}
//
//////////////////////////////////////////////////////////////////////////////
