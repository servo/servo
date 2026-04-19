// Copyright (C) 2018 Ujjwal Sharma. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-number-format-functions
description: >
  Tests that the default value for the argument of
  Intl.NumberFormat.prototype.format (value) is undefined.
info: |
  11.1.4 Number Format Functions

  3. If value is not provided, let value be undefined.
  4. Let x be ? ToNumber(value).
---*/

const nf = new Intl.NumberFormat();

// In most locales this is string "NaN", but there are exceptions, cf. "ليس رقم"
// in Arabic, "epäluku" in Finnish, "не число" in Russian, "son emas" in Uzbek etc.
const resultNaN = nf.format(NaN);

assert.sameValue(nf.format(), resultNaN);
assert.sameValue(nf.format(undefined), resultNaN);
