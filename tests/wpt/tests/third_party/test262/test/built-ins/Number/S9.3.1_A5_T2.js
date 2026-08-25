// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: |
    The MV of StrDecimalLiteral::: - StrUnsignedDecimalLiteral is the negative
    of the MV of StrUnsignedDecimalLiteral. (the negative of this 0 is also 0)
es5id: 9.3.1_A5_T2
description: Compare Number('-[or +]any_number') with -[or without -]any_number)
---*/
assert.sameValue(Number("1"), 1, 'Number("1") must return 1');
assert.sameValue(Number("+1"), 1, 'Number("+1") must return 1');
assert.sameValue(Number("-1"), -1, 'Number("-1") must return -1');
assert.sameValue(Number("2"), 2, 'Number("2") must return 2');
assert.sameValue(Number("+2"), 2, 'Number("+2") must return 2');
assert.sameValue(Number("-2"), -2, 'Number("-2") must return -2');
assert.sameValue(Number("3"), 3, 'Number("3") must return 3');
assert.sameValue(Number("+3"), 3, 'Number("+3") must return 3');
assert.sameValue(Number("-3"), -3, 'Number("-3") must return -3');
assert.sameValue(Number("4"), 4, 'Number("4") must return 4');
assert.sameValue(Number("+4"), 4, 'Number("+4") must return 4');
assert.sameValue(Number("-4"), -4, 'Number("-4") must return -4');
assert.sameValue(Number("5"), 5, 'Number("5") must return 5');
assert.sameValue(Number("+5"), 5, 'Number("+5") must return 5');
assert.sameValue(Number("-5"), -5, 'Number("-5") must return -5');
assert.sameValue(Number("6"), 6, 'Number("6") must return 6');
assert.sameValue(Number("+6"), 6, 'Number("+6") must return 6');
assert.sameValue(Number("-6"), -6, 'Number("-6") must return -6');
assert.sameValue(Number("7"), 7, 'Number("7") must return 7');
assert.sameValue(Number("+7"), 7, 'Number("+7") must return 7');
assert.sameValue(Number("-7"), -7, 'Number("-7") must return -7');
assert.sameValue(Number("8"), 8, 'Number("8") must return 8');
assert.sameValue(Number("+8"), 8, 'Number("+8") must return 8');
assert.sameValue(Number("-8"), -8, 'Number("-8") must return -8');
assert.sameValue(Number("9"), 9, 'Number("9") must return 9');
assert.sameValue(Number("+9"), 9, 'Number("+9") must return 9');
assert.sameValue(Number("-9"), -9, 'Number("-9") must return -9');
