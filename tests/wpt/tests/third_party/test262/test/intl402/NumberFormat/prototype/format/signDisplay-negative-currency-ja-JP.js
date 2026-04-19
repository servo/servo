// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [ja-JP]
features: [Intl.NumberFormat-v3]
---*/

const nf = new Intl.NumberFormat("ja-JP", { style: "currency", currency: "USD", currencySign: "accounting", signDisplay: "negative" });
assert.sameValue(nf.format(-987), "($987.00)");
assert.sameValue(nf.format(-0.0001), "$0.00");
assert.sameValue(nf.format(-0), "$0.00");
assert.sameValue(nf.format(0), "$0.00");
assert.sameValue(nf.format(0.0001), "$0.00");
assert.sameValue(nf.format(987), "$987.00");
