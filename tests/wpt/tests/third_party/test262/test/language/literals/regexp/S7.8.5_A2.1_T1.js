// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    RegularExpressionChar :: NonTerminator but not \ or /,
    RegularExpressionFlags :: [empty]
es5id: 7.8.5_A2.1_T1
description: Without eval
---*/

//CHECK#1
if (/1a/.source !== "1a") {
  throw new Test262Error('#1: /1a/');
}   

//CHECK#2
if (/aa/.source !== "aa") {
  throw new Test262Error('#2: /aa/');
}

//CHECK#3
if (/,;/.source !== ",;") {
  throw new Test262Error('#3: /,;/');
}

//CHECK#4
if (/  /.source !== "  ") {
  throw new Test262Error('#4: /  /');
}      

//CHECK#5
if (/a\u0041/.source !== "a\\u0041") {
  throw new Test262Error('#5: /a\\u0041/');
}
