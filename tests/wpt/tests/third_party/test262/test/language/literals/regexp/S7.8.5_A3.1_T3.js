// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "RegularExpressionFlags :: IdentifierPart"
es5id: 7.8.5_A3.1_T3
description: "IdentifierPart :: m"
---*/

//CHECK#1
var regexp = /(?:)/m; 
if (regexp.global !== false) {
  throw new Test262Error('#1: var regexp = /(?:)/g; regexp.global === false. Actual: ' + (regexp.global));
}

//CHECK#2 
if (regexp.ignoreCase !== false) {
  throw new Test262Error('#2: var regexp = /(?:)/g; regexp.ignoreCase === false. Actual: ' + (regexp.ignoreCase));
}

//CHECK#3
if (regexp.multiline !== true) {
  throw new Test262Error('#3: var regexp = /(?:)/g; regexp.multiline === true. Actual: ' + (regexp.multiline));
}
