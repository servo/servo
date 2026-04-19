// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "CharacterEscapeSequnce :: SingleEscapeSequence"
es5id: 7.8.4_A4.1_T1
description: "SingleEscapeSequence :: one of b f n r t v"
---*/

//CHECK#1
if (String.fromCharCode(0x0008) !== "\b") {
  throw new Test262Error('#1: String.fromCharCode(0x0008) === "\\b"');
}

//CHECK#2
if (String.fromCharCode(0x0009) !== "\t") {
  throw new Test262Error('#2: String.fromCharCode(0x0009) === "\\t"');
}

//CHECK#3
if (String.fromCharCode(0x000A) !== "\n") {
  throw new Test262Error('#3: String.fromCharCode(0x000A) === "\\n"');
}

//CHECK#4
if (String.fromCharCode(0x000B) !== "\v") {
  throw new Test262Error('#4: String.fromCharCode(0x000B) === "\\v"');
}

//CHECK#5
if (String.fromCharCode(0x000C) !== "\f") {
  throw new Test262Error('#5: String.fromCharCode(0x000C) === "\\f"');
}

//CHECK#6
if (String.fromCharCode(0x000D) !== "\r") {
  throw new Test262Error('#6: String.fromCharCode(0x000D) === "\\r"');
}
