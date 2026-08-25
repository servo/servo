// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: VERTICAL TAB (U+000B) may occur within strings
es5id: 7.2_A2.2_T1
description: Use VERTICAL TAB(\u000B and \v)
---*/

// CHECK#1
if (eval("'\u000Bstr\u000Bing\u000B'") !== "\u000Bstr\u000Bing\u000B") {
  throw new Test262Error('#1: eval("\'\\u000Bstr\\u000Bing\\u000B\'") === "\\u000Bstr\\u000Bing\\u000B"');
}

//CHECK#2
if (eval("'\vstr\ving\v'") !== "\vstr\ving\v") {
  throw new Test262Error('#2: eval("\'\\vstr\\ving\\v\'") === "\\vstr\\ving\\v"');
}
