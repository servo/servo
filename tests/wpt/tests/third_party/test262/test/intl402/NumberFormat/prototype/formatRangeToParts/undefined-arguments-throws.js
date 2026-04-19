// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.NumberFormat.prototype.formatRangeToParts
description: >
  "formatRangeToParts" basic tests when arguments are undefined throw a TypeError exception.
info: |
  Intl.NumberFormat.prototype.formatRangeToParts ( start, end )
  (...)
  3. If start is undefined or end is undefined, throw a TypeError exception.
features: [Intl.NumberFormat-v3]
---*/

const nf = new Intl.NumberFormat();

//  If arguments are undefined throw a TypeError exception.
assert.throws(TypeError, () => { nf.formatRangeToParts(undefined, 23) });
assert.throws(TypeError, () => { nf.formatRangeToParts(1,undefined) });
assert.throws(TypeError, () => { nf.formatRangeToParts(undefined, undefined)});
