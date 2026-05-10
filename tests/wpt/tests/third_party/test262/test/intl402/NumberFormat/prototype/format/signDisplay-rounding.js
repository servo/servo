// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [en-US]
features: [Intl.NumberFormat-unified]
---*/


const fmt = new Intl.NumberFormat("en-US", {
  maximumFractionDigits: 1,
  signDisplay: "exceptZero"
});

assert.sameValue(fmt.format(0.01), "0");
assert.sameValue(fmt.format(-0.01), "0");
