// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 6.1
description: Test for handling of supplementary characters
---*/

var chars = "𐒠";  // Single Unicode character at codepoint \u{104A0}
if(chars.length !== 2) {
    throw new Test262Error("A character outside the BMP (Unicode CodePoint > 0xFFFF) should consume two code units");
}
if(chars.charCodeAt(0) !== 0xD801) {
    throw new Test262Error("First code unit of surrogate pair for 0x104A0 should be 0xD801");
}

if(chars.charCodeAt(1) !== 0xDCA0) {
    throw new Test262Error("Second code unit of surrogate pair for 0x104A0 should be 0xDCA0");
}
