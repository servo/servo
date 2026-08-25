// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    If the length of S is at least 2 and the first two characters of S
    are either 0x or 0X, then remove the first two characters from S and let R = 16
esid: sec-parseint-string-radix
description: ": 0X"
---*/

assert.sameValue(parseInt("0X0", 0), parseInt("0", 16), 'parseInt("0X0", 0) must return the same value returned by parseInt("0", 16)');
assert.sameValue(parseInt("0X1"), parseInt("1", 16), 'parseInt("0X1") must return the same value returned by parseInt("1", 16)');
assert.sameValue(parseInt("0X2"), parseInt("2", 16), 'parseInt("0X2") must return the same value returned by parseInt("2", 16)');
assert.sameValue(parseInt("0X3"), parseInt("3", 16), 'parseInt("0X3") must return the same value returned by parseInt("3", 16)');
assert.sameValue(parseInt("0X4"), parseInt("4", 16), 'parseInt("0X4") must return the same value returned by parseInt("4", 16)');
assert.sameValue(parseInt("0X5"), parseInt("5", 16), 'parseInt("0X5") must return the same value returned by parseInt("5", 16)');
assert.sameValue(parseInt("0X6"), parseInt("6", 16), 'parseInt("0X6") must return the same value returned by parseInt("6", 16)');
assert.sameValue(parseInt("0X7"), parseInt("7", 16), 'parseInt("0X7") must return the same value returned by parseInt("7", 16)');
assert.sameValue(parseInt("0X8"), parseInt("8", 16), 'parseInt("0X8") must return the same value returned by parseInt("8", 16)');
assert.sameValue(parseInt("0X9"), parseInt("9", 16), 'parseInt("0X9") must return the same value returned by parseInt("9", 16)');
assert.sameValue(parseInt("0XA"), parseInt("A", 16), 'parseInt("0XA") must return the same value returned by parseInt("A", 16)');
assert.sameValue(parseInt("0XB"), parseInt("B", 16), 'parseInt("0XB") must return the same value returned by parseInt("B", 16)');
assert.sameValue(parseInt("0XC"), parseInt("C", 16), 'parseInt("0XC") must return the same value returned by parseInt("C", 16)');
assert.sameValue(parseInt("0XD"), parseInt("D", 16), 'parseInt("0XD") must return the same value returned by parseInt("D", 16)');
assert.sameValue(parseInt("0XE"), parseInt("E", 16), 'parseInt("0XE") must return the same value returned by parseInt("E", 16)');
assert.sameValue(parseInt("0XF"), parseInt("F", 16), 'parseInt("0XF") must return the same value returned by parseInt("F", 16)');
assert.sameValue(parseInt("0XE"), parseInt("E", 16), 'parseInt("0XE") must return the same value returned by parseInt("E", 16)');
assert.sameValue(parseInt("0XABCDEF"), parseInt("ABCDEF", 16), 'parseInt("0XABCDEF") must return the same value returned by parseInt("ABCDEF", 16)');
