// Copyright (C) 2020 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-quantifier
description: >
  MV of DecimalDigits evaluates to 2 ** 53 - 1.
  (although DecimalDigits could be arbitrary large integer)
info: |
  Quantifier

  The production QuantifierPrefix :: { DecimalDigits } evaluates as follows:

  1. Let i be the MV of DecimalDigits (see 11.8.3).
  2. Return the two results i and i.

  The production QuantifierPrefix :: { DecimalDigits, } evaluates as follows:

  1. Let i be the MV of DecimalDigits.
  2. Return the two results i and ∞.

  The production QuantifierPrefix :: { DecimalDigits, DecimalDigits } evaluates as follows:

  1. Let i be the MV of the first DecimalDigits.
  2. Let j be the MV of the second DecimalDigits.
  3. Return the two results i and j.
---*/

var re1 = new RegExp("b{" + Number.MAX_SAFE_INTEGER + "}", "u");
assert(!re1.test(""));

var re2 = new RegExp("b{" + Number.MAX_SAFE_INTEGER + ",}?");
assert(!re2.test("a"));

var re3 = new RegExp("b{" + Number.MAX_SAFE_INTEGER + "," + Number.MAX_SAFE_INTEGER + "}");
assert(!re3.test("b"));
