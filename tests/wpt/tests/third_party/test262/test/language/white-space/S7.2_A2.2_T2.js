// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: VERTICAL TAB (U+000B) may occur within strings
es5id: 7.2_A2.2_T2
description: Use real VERTICAL TAB
---*/

//CHECK#1
if ("string" !== "\u000Bstr\u000Bing\u000B") {
  throw new Test262Error('#1: "string" === "\\u000Bstr\\u000Bing\\u000B"');
}
