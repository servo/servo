// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Single line comments are terminated by the LINE FEED (U+000A) character
esid: sec-line-terminators
es5id: 7.3_A3.1_T1
description: Insert LINE FEED (\u000A) into single line comment
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

// single line
throw new Test262Error();
