// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: Checks handling of the compactDisplay option to the NumberFormat constructor.
info: |
  Intl.NumberFormat ( [ locales [ , options ] ] )

  1. Let _compactDisplay_ be ? GetOption(_options_, *"compactDisplay"*,
     ~string~, « *"short"*, *"long"* », *"short"*).
  1. If _notation_ is *"compact"*, then
    1. Set _numberFormat_.[[CompactDisplay]] to _compactDisplay_.

features: [Intl.NumberFormat-unified]
---*/

for (const notation of [undefined, "standard", "scientific", "engineering"]) {
  for (const value of [undefined, "short", "long"]) {
    const nf = new Intl.NumberFormat([], { notation, compactDisplay: value });
    const resolvedOptions = nf.resolvedOptions();
    assert.sameValue("compactDisplay" in resolvedOptions, false);
    assert.sameValue(resolvedOptions.compactDisplay, undefined);
  }
}
