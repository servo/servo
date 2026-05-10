// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-literals-numeric-literals
description: Mathematical value for LegacyOctalIntegerLiteral
info: |
    NumericLiteral ::
      DecimalLiteral
      BinaryIntegerLiteral
      OctalIntegerLiteral
      HexIntegerLiteral
      LegacyOctalIntegerLiteral

     LegacyOctalIntegerLiteral ::
       0 OctalDigit
       LegacyOctalIntegerLiteral OctalDigit
flags: [noStrict]
---*/

// LegacyOctalIntegerLiteral ::
//   0 OctalDigit
assert.sameValue(00, 0, '00');
assert.sameValue(01, 1, '01');
assert.sameValue(02, 2, '02');
assert.sameValue(03, 3, '03');
assert.sameValue(04, 4, '04');
assert.sameValue(05, 5, '05');
assert.sameValue(06, 6, '06');
assert.sameValue(07, 7, '07');

// LegacyOctalIntegerLiteral ::
//   LegacyOctalIntegerLiteral OctalDigit
assert.sameValue(000, 0, '000');
assert.sameValue(001, 1, '001');
assert.sameValue(002, 2, '002');
assert.sameValue(003, 3, '003');
assert.sameValue(004, 4, '004');
assert.sameValue(005, 5, '005');
assert.sameValue(006, 6, '006');
assert.sameValue(007, 7, '007');

assert.sameValue(070, 56);
assert.sameValue(071, 57);
assert.sameValue(072, 58);
assert.sameValue(073, 59);
assert.sameValue(074, 60);
assert.sameValue(075, 61);
assert.sameValue(076, 62);
assert.sameValue(077, 63);
