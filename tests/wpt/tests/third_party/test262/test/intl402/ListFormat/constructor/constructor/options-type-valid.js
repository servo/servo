// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks handling of valid values for the style option to the ListFormat constructor.
info: |
    InitializeListFormat (listFormat, locales, options)
    7. Let type be GetOption(options, "type", "string", « "conjunction", "disjunction", "unit" », "conjunction").
    8. Set listFormat.[[Type]] to type.
features: [Intl.ListFormat]
---*/

const validOptions = [
  [undefined, "conjunction"],
  ["conjunction", "conjunction"],
  ["disjunction", "disjunction"],
  ["unit", "unit"],
  [{ toString() { return "unit"; } }, "unit"],
];

for (const [validOption, expected] of validOptions) {
  const lf = new Intl.ListFormat([], {"type": validOption});
  const resolvedOptions = lf.resolvedOptions();
  assert.sameValue(resolvedOptions.type, expected);
}
