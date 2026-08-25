// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: LF (U+000A)"
---*/

assert.sameValue(parseInt("\u000A1"), parseInt("1"), 'parseInt("\\u000A1") must return the same value returned by parseInt("1")');
assert.sameValue(parseInt("\u000A\u000A-1"), parseInt("-1"), 'parseInt("\\u000A\\u000A-1") must return the same value returned by parseInt("-1")');

//CHECK#3
assert.sameValue(parseInt("\u000A"), NaN, 'parseInt("\\u000A") must return NaN');
