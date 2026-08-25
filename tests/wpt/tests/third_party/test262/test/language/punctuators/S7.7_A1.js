// Copyright 2009 the Sputnik authors.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
info: Correct interpretation of all punctuators
es5id: 7.7_A1
description: Using all punctuators
---*/

this.nan = NaN;

//CHECK#1
  ({});[];
  this.nan;
  1 < 2 > 3 <= 4 >= 5 == 6 != 7 === 8 !== 9;
  1 + 2 - 3 * 4 % 5 / 6 << 7 >> 8 >>> 9;
  this.nan++; ++this.nan; this.nan--; --this.nan;
  1 & 2 | 3 ^ 4 && !5 || ~6;
  1 ? 2 : 3;
  this.nan = 1; this.nan += 2; this.nan -= 3; this.nan *= 4; this.nan /= 5;
  this.nan %= 6; this.nan <<= 7; this.nan >>= 8; this.nan >>>= 9;
  this.nan &= 1; this.nan |= 2; this.nan ^= 3;
