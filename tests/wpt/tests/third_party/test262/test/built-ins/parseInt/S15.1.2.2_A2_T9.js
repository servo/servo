// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: PS (U+2029)"
---*/

assert.sameValue(
  parseInt("\u20291"),
  parseInt("1"),
  'parseInt("\\u20291") must return the same value returned by parseInt("1")'
);

assert.sameValue(
  parseInt("\u2029\u2029-1"),
  parseInt("-1"),
  'parseInt("\\u2029\\u2029-1") must return the same value returned by parseInt("-1")'
);

//CHECK#3
assert.sameValue(parseInt("\u2029"), NaN, 'parseInt("\\u2029") must return NaN');
