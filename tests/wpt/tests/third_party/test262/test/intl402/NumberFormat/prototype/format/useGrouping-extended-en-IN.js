// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: |
  Checks handling of the useGrouping option to the NumberFormat constructor.
locale: [en-IN]
features: [Intl.NumberFormat-v3]
---*/

var nf;

nf = new Intl.NumberFormat('en-IN', {useGrouping: 'always'});

assert.sameValue(nf.format(100), '100', '"always"');
assert.sameValue(nf.format(1000), '1,000', '"always"');
assert.sameValue(nf.format(10000), '10,000', '"always"');
assert.sameValue(nf.format(100000), '1,00,000', '"always"');

nf = new Intl.NumberFormat('en-IN', {useGrouping: 'min2'});

assert.sameValue(nf.format(100), '100', '"min2"');
assert.sameValue(nf.format(1000), '1000', '"min2"');
assert.sameValue(nf.format(10000), '10,000', '"min2"');
assert.sameValue(nf.format(100000), '1,00,000', '"min2"');

nf = new Intl.NumberFormat('en-IN', {notation: 'compact'});

assert.sameValue(nf.format(100), '100', 'notation: "compact"');
assert.sameValue(nf.format(1000), '1K', 'notation: "compact"');
assert.sameValue(nf.format(10000), '10K', 'notation: "compact"');
assert.sameValue(nf.format(100000), '1L', 'notation: "compact"');
