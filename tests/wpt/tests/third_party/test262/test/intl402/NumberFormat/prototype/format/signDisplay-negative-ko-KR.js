// Copyright 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.format
description: Checks handling of the signDisplay option to the NumberFormat constructor.
locale: [ko-KR]
features: [Intl.NumberFormat-v3]
---*/

const nf = new Intl.NumberFormat("ko-KR", {signDisplay: "negative"});
assert.sameValue(nf.format(-Infinity), "-∞", "-Infinity");
assert.sameValue(nf.format(-987), "-987", "-987");
assert.sameValue(nf.format(-0.0001), "0", "-0.0001");
assert.sameValue(nf.format(-0), "0", "-0");
assert.sameValue(nf.format(0), "0", "0");
assert.sameValue(nf.format(0.0001), "0", "0.0001");
assert.sameValue(nf.format(987), "987", "987");
assert.sameValue(nf.format(Infinity), "∞", "Infinity");
assert.sameValue(nf.format(NaN), "NaN", "NaN");
