// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Multi line comment can contain LINE SEPARATOR (U+2028)
esid: sec-line-terminators
es5id: 7.3_A5.3
description: Insert LINE SEPARATOR (U+2028) into multi line comment
negative:
  phase: runtime
  type: Test262Error
---*/

// Because this test concerns the interpretation of non-executable character
// sequences within ECMAScript source code, special care must be taken to
// ensure that executable code is evaluated as expected.
//
// Express the intended behavior by intentionally throwing an error; this
// guarantees that test runners will only consider the test "passing" if
// executable sequences are correctly interpreted as such.

var x = 0;

/* x = 1; */

if (x === 0) {
  throw new Test262Error();
}
