// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-number-format-functions
description: >
  Intl.NumberFormat.prototype.format converts its argument (called value) to a
  number using ToIntlMathematicalValue.
features: [Intl.NumberFormat-v3]
locale: [en-US]
---*/

var nf = new Intl.NumberFormat('en-US', {maximumFractionDigits: 20});

// The value 100,000 should only be interpreted as infinity if the input is the
// string "Infinity".
assert.sameValue(nf.format('100000'), '100,000');
// The value -100,000 should only be interpreted as negative infinity if the
// input is the string "-Infinity".
assert.sameValue(nf.format('-100000'), '-100,000');

assert.sameValue(nf.format('1.0000000000000001'), '1.0000000000000001');
assert.sameValue(nf.format('-1.0000000000000001'), '-1.0000000000000001');
assert.sameValue(nf.format('987654321987654321'), '987,654,321,987,654,321');
assert.sameValue(nf.format('-987654321987654321'), '-987,654,321,987,654,321');
