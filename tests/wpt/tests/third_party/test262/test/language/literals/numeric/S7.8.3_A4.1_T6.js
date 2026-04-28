// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: "DecimalLiteral :: ExponentPart is incorrect"
es5id: 7.8.3_A4.1_T6
description: "ExponentPart :: E DecimalDigits"
---*/

assert.throws(ReferenceError, function() {
  E+1
});
