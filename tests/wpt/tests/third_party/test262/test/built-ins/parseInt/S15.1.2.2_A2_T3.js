// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Operator remove leading StrWhiteSpaceChar
esid: sec-parseint-string-radix
description: "StrWhiteSpaceChar :: NBSB (U+00A0)"
---*/

assert.sameValue(parseInt("\u00A01"), parseInt("1"), 'parseInt("\\u00A01") must return the same value returned by parseInt("1")');
assert.sameValue(parseInt("\u00A0\u00A0-1"), parseInt("-1"), 'parseInt("\\u00A0\\u00A0-1") must return the same value returned by parseInt("-1")');

//CHECK#3
assert.sameValue(parseInt("\u00A0"), NaN, 'parseInt("\\u00A0") must return NaN');
