// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: Checks handling of the notation option to the NumberFormat constructor.
info: |
    InitializeNumberFormat ( numberFormat, locales, options )

    16. Let notation be ? GetOption(options, "notation", "string", « "standard", "scientific", "engineering", "compact" », "standard").
    17. Set numberFormat.[[Notation]] to notation.

features: [Intl.NumberFormat-unified]
---*/

const values = [
  [undefined, "standard"],
  ["standard"],
  ["scientific"],
  ["engineering"],
  ["compact"],
];

for (const [value, expected = value] of values) {
  const nf = new Intl.NumberFormat([], {
    notation: value,
  });
  const resolvedOptions = nf.resolvedOptions();
  assert.sameValue("notation" in resolvedOptions, true);
  assert.sameValue(resolvedOptions.notation, expected);
}
