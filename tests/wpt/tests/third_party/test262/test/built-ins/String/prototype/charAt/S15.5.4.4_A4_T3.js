// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If pos is a value of Number type that is an integer, then the result of
    x.charAt(pos) is equal to the result of x.substring(pos, pos+1)
es5id: 15.5.4.4_A4_T3
description: >
    Compare results of x.charAt(pos) and x.substring(pos, pos+1), wheb
    pos is bigger string length
---*/

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
for (var i = 6; i < 8; i++) {
  if ("ABC\u0041\u0042\u0043".charAt(i) !== "\u0041\u0042\u0043ABC".substring(i, i + 1)) {
    throw new Test262Error('#' + (i - 5) + ': "ABC\\u0041\\u0042\\u0043".charAt(' + i + ') === "\\u0041\\u0042\\u0043ABC".substring(' + i + ', ' + (i + 1) + '). Actual: "ABC\\u0041\\u0042\\u0043".charAt(' + i + ') ===' + ("ABC\u0041\u0042\u0043".charAt(i)));
  }
}

//
//////////////////////////////////////////////////////////////////////////////
