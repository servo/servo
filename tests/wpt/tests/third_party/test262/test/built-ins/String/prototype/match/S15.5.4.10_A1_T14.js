// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: String.prototype.match (regexp)
es5id: 15.5.4.10_A1_T14
description: >
    Call match (regexp) function with RegExp object as argument from
    string
---*/

var __reg = new RegExp("77");

//////////////////////////////////////////////////////////////////////////////
//CHECK#1
if ("ABB\u0041BABAB\u0037\u0037BBAA".match(__reg)[0] !== "77") {
  throw new Test262Error('#1: var __reg = new RegExp("77"); "ABB\\u0041BABAB\\u0037\\u0037BBAA".match(__reg)[0] === "77". Actual: ' + ("ABB\u0041BABAB\u0037\u0037BBAA".match(__reg)[0]));
}
//
//////////////////////////////////////////////////////////////////////////////
