// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-bigint.prototype.tostring
description: >
  Letters a-z are used for digits with values 10 through 35
info: |
  6. Return the String representation of this Number value using
  the radix specified by radixNumber. Letters a-z are used for
  digits with values 10 through 35. The precise algorithm is
  implementation-dependent, however the algorithm should be a
  generalization of that specified in 6.1.6.2.23.
features: [BigInt]
---*/

for (let radix = 11; radix <= 36; radix++) {
  for (let i = 10n; i < radix; i++) {
    assert.sameValue(i.toString(radix), String.fromCharCode(Number(i + 87n)));
  }
}
