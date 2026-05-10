// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: LS (U+2028)"
---*/

assert.sameValue(parseInt("\u20281"), parseInt("1"), 'parseInt("\\u20281") must return the same value returned by parseInt("1")');
assert.sameValue(parseInt("\u2028\u2028-1"), parseInt("-1"), 'parseInt("\\u2028\\u2028-1") must return the same value returned by parseInt("-1")');

//CHECK#3
assert.sameValue(parseInt("\u2028"), NaN, 'parseInt("\\u2028") must return NaN');
