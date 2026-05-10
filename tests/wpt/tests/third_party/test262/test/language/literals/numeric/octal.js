// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 11.8.3.1
description: Mathematical value of valid octal integer literals
info: |
    The MV of StrNumericLiteral ::: OctalIntegerLiteral is the MV of
    OctalIntegerLiteral.
    The MV of OctalIntegerLiteral :: 0o OctalDigits is the MV of OctalDigits.
    The MV of OctalIntegerLiteral :: 0O OctalDigits is the MV of OctalDigits.
    The MV of OctalDigits :: OctalDigit is the MV of OctalDigit.
    The MV of OctalDigits :: OctalDigits OctalDigit is (the MV of OctalDigits ×
    8) plus the MV of OctalDigit.
---*/

assert.sameValue(0o0, 0, 'lower-case head');
assert.sameValue(0O0, 0, 'upper-case head');
assert.sameValue(0o00, 0, 'lower-case head with leading zeros');
assert.sameValue(0O00, 0, 'upper-case head with leading zeros');

assert.sameValue(0o1, 1, 'lower-case head');
assert.sameValue(0O1, 1, 'upper-case head');
assert.sameValue(0o01, 1, 'lower-case head with leading zeros');
assert.sameValue(0O01, 1, 'upper-case head with leading zeros');

assert.sameValue(0o7, 7, 'lower-case head');
assert.sameValue(0O7, 7, 'upper-case head');
assert.sameValue(0o07, 7, 'lower-case head with leading zeros');
assert.sameValue(0O07, 7, 'upper-case head with leading zeros');

assert.sameValue(0o10, 8, 'lower-case head');
assert.sameValue(0O10, 8, 'upper-case head');
assert.sameValue(0o010, 8, 'lower-case head with leading zeros');
assert.sameValue(0O010, 8, 'upper-case head with leading zeros');

assert.sameValue(0o11, 9, 'lower-case head');
assert.sameValue(0O11, 9, 'upper-case head');
assert.sameValue(0o011, 9, 'lower-case head with leading zeros');
assert.sameValue(0O011, 9, 'upper-case head with leading zeros');

assert.sameValue(0o77, 63, 'lower-case head');
assert.sameValue(0O77, 63, 'upper-case head');
assert.sameValue(0o077, 63, 'lower-case head with leading zeros');
assert.sameValue(0O077, 63, 'upper-case head with leading zeros');
