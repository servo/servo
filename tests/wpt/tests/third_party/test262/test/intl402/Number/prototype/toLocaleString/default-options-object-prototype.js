// Copyright (C) 2017 Daniel Ehrenberg. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: >
  Monkey-patching Object.prototype does not change the default
  options for NumberFormat as a null prototype is used.
info: |
  Intl.NumberFormat ( [ locales [ , options ] ] )

    1. If _options_ is *undefined*, then
      1. Let _options_ be ObjectCreate(*null*).
---*/

if (new Intl.NumberFormat("en").resolvedOptions().locale === "en") {
  Object.prototype.maximumFractionDigits = 1;
  assert.sameValue(1.23.toLocaleString("en"), "1.23");
}
