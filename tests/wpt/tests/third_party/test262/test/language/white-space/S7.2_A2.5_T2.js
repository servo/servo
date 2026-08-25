// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: NO-BREAK SPACE (U+00A0) may occur within strings
es5id: 7.2_A2.5_T2
description: Use real NO-BREAK SPACE
---*/

//CHECK#1
if (" str ing " !== "\u00A0str\u00A0ing\u00A0") {
  throw new Test262Error('#1: " str ing " === "\\u00A0str\\u00A0ing\\u00A0"');
}
