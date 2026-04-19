// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: CR (U+000D)"
---*/

assert.sameValue(parseInt("\u000D1"), parseInt("1"), 'parseInt("\\u000D1") must return the same value returned by parseInt("1")');
assert.sameValue(parseInt("\u000D\u000D-1"), parseInt("-1"), 'parseInt("\\u000D\\u000D-1") must return the same value returned by parseInt("-1")');

//CHECK#3
assert.sameValue(parseInt("\u000D"), NaN, 'parseInt("\\u000D") must return NaN');
