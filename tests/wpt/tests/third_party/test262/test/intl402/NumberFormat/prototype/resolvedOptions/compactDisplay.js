// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.resolvedoptions
description: Verifies the existence of the compactDisplay property for the object returned by resolvedOptions().
features: [Intl.NumberFormat-unified]
---*/

for (const notation of [undefined, "standard", "scientific", "engineering"]) {
  const options = new Intl.NumberFormat([], {
    notation,
    compactDisplay: "long",
  }).resolvedOptions();
  assert.sameValue("compactDisplay" in options, false, `There should be no compactDisplay property with notation=${notation}`);
  assert.sameValue(options.compactDisplay, undefined, `The compactDisplay property should be undefined with notation=${notation}`);
}

const options = new Intl.NumberFormat([], {
  notation: "compact",
  compactDisplay: "long",
}).resolvedOptions();
assert.sameValue("compactDisplay" in options, true, "There should be a compactDisplay property with notation=compact");
assert.sameValue(options.compactDisplay, "long", "The compactDisplay property should be defined with notation=compact");
