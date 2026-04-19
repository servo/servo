// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the compactDisplay option to the NumberFormat constructor.
locale: [en-US]
features: [Intl.NumberFormat-unified]
---*/

const nfShort = new Intl.NumberFormat("en-US", {
  notation: "compact",
  compactDisplay: "short",
});
assert.sameValue(nfShort.format(987654321), "988M");
assert.sameValue(nfShort.format(98765432), "99M");
assert.sameValue(nfShort.format(98765), "99K");
assert.sameValue(nfShort.format(9876), "9.9K");
assert.sameValue(nfShort.format(159), "159");
assert.sameValue(nfShort.format(15.9), "16");
assert.sameValue(nfShort.format(1.59), "1.6");
assert.sameValue(nfShort.format(0.159), "0.16");
assert.sameValue(nfShort.format(0.0159), "0.016");
assert.sameValue(nfShort.format(0.00159), "0.0016");
assert.sameValue(nfShort.format(-Infinity), "-∞");
assert.sameValue(nfShort.format(Infinity), "∞");
assert.sameValue(nfShort.format(NaN), "NaN");

const nfLong = new Intl.NumberFormat("en-US", {
  notation: "compact",
  compactDisplay: "long",
});
assert.sameValue(nfLong.format(987654321), "988 million");
assert.sameValue(nfLong.format(98765432), "99 million");
assert.sameValue(nfLong.format(98765), "99 thousand");
assert.sameValue(nfLong.format(9876), "9.9 thousand");
assert.sameValue(nfLong.format(159), "159");
assert.sameValue(nfLong.format(15.9), "16");
assert.sameValue(nfLong.format(1.59), "1.6");
assert.sameValue(nfLong.format(0.159), "0.16");
assert.sameValue(nfLong.format(0.0159), "0.016");
assert.sameValue(nfLong.format(0.00159), "0.0016");
assert.sameValue(nfLong.format(-Infinity), "-∞");
assert.sameValue(nfLong.format(Infinity), "∞");
assert.sameValue(nfLong.format(NaN), "NaN");
