// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-setnumberformatunitoptions
description: Checks handling of valid values for the numeric option to the RelativeTimeFormat constructor.
info: |
    SetNumberFormatUnitOptions ( intlObj, options )

    3. Let style be ? GetOption(options, "style", "string", « "decimal", "percent", "currency", "unit" », "decimal").
    4. Set intlObj.[[Style]] to style.

features: [Intl.NumberFormat-unified]
---*/

const validOptions = [
  [undefined, "decimal"],
  ["unit", "unit"],
  [{ toString() { return "unit"; } }, "unit"],
];

for (const [validOption, expected] of validOptions) {
  const nf = new Intl.NumberFormat([], {"style": validOption, "unit": "gigabit"});
  const resolvedOptions = nf.resolvedOptions();
  assert.sameValue(resolvedOptions.style, expected);
}
