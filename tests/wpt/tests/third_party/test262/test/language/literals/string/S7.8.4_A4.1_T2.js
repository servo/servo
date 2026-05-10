// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "CharacterEscapeSequnce :: SingleEscapeSequence"
es5id: 7.8.4_A4.1_T2
description: "SingleEscapeSequence :: one of ' \" \\"
---*/

//CHECK#1
if (String.fromCharCode(0x0027) !== "\'") {
  throw new Test262Error('#1: String.fromCharCode(0x0027) === "\\\'"');
}

//CHECK#2
if (String.fromCharCode(0x0022) !== '\"') {
  throw new Test262Error('#2: String.fromCharCode(0x0027) === \'\\\"\'');
}

//CHECK#3
if (String.fromCharCode(0x005C) !== "\\") {
  throw new Test262Error('#3: String.fromCharCode(0x005C) === "\\\"');
}

//CHECK#4
if ("\'" !== "'") {
  throw new Test262Error('#4: "\'" === "\\\'"');
}

//CHECK#5
if ('\"' !== '"') {
  throw new Test262Error('#5: \'\"\' === \'\\\"\'');
}
