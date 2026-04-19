// Copyright (C) 2015 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es6id: 20.1.1.1
description: Mathematical value of valid octal integer literals
info: |
    20.1.1.1 Number ( [ value ] )

    When Number is called with argument number, the following steps are taken:

    1. If no arguments were passed to this function invocation, let n be +0.
    2. Else, let n be ToNumber(value).

    [...]

    7.1.3.1 ToNumber Applied to the String Type

    All grammar symbols not explicitly defined above have the definitions used
    in the Lexical Grammar for numeric literals (11.8.3)

    [...]

    The MV of OctalIntegerLiteral :: 0o OctalDigits is the MV of OctalDigits.
    The MV of OctalIntegerLiteral :: 0O OctalDigits is the MV of OctalDigits.
    The MV of OctalDigits :: OctalDigit is the MV of OctalDigit.
    The MV of OctalDigits :: OctalDigits OctalDigit is (the MV of OctalDigits ×
    8) plus the MV of OctalDigit.
---*/

assert.sameValue(Number('0o0'), 0, 'lower-case head');
assert.sameValue(Number('0O0'), 0, 'upper-case head');
assert.sameValue(Number('0o00'), 0, 'lower-case head with leading zeros');
assert.sameValue(Number('0O00'), 0, 'upper-case head with leading zeros');

assert.sameValue(Number('0o1'), 1, 'lower-case head');
assert.sameValue(Number('0O1'), 1, 'upper-case head');
assert.sameValue(Number('0o01'), 1, 'lower-case head with leading zeros');
assert.sameValue(Number('0O01'), 1, 'upper-case head with leading zeros');

assert.sameValue(Number('0o7'), 7, 'lower-case head');
assert.sameValue(Number('0O7'), 7, 'upper-case head');
assert.sameValue(Number('0o07'), 7, 'lower-case head with leading zeros');
assert.sameValue(Number('0O07'), 7, 'upper-case head with leading zeros');

assert.sameValue(Number('0o10'), 8, 'lower-case head');
assert.sameValue(Number('0O10'), 8, 'upper-case head');
assert.sameValue(Number('0o010'), 8, 'lower-case head with leading zeros');
assert.sameValue(Number('0O010'), 8, 'upper-case head with leading zeros');

assert.sameValue(Number('0o11'), 9, 'lower-case head');
assert.sameValue(Number('0O11'), 9, 'upper-case head');
assert.sameValue(Number('0o011'), 9, 'lower-case head with leading zeros');
assert.sameValue(Number('0O011'), 9, 'upper-case head with leading zeros');

assert.sameValue(Number('0o77'), 63, 'lower-case head');
assert.sameValue(Number('0O77'), 63, 'upper-case head');
assert.sameValue(Number('0o077'), 63, 'lower-case head with leading zeros');
assert.sameValue(Number('0O077'), 63, 'upper-case head with leading zeros');
