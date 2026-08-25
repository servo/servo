// Copyright 2019 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: >
    Checks handling of the options argument to the DateTimeFormat constructor.
info: |
    CreateDateTimeFormat ( dateTimeFormat, locales, options, required, defaults )

    ...
    39. Let dateStyle be ? GetOption(options, "dateStyle", "string", « "full", "long", "medium", "short" », undefined).
    40. If dateStyle is not undefined, set dateTimeFormat.[[DateStyle]] to dateStyle.
features: [Intl.DateTimeFormat-datetimestyle]
---*/


const validOptions = [
  [undefined, undefined],
  ["full", "full"],
  ["long", "long"],
  ["medium", "medium"],
  ["short", "short"],
  [{ toString() { return "full"; } }, "full"],
  [{ valueOf() { return "long"; }, toString: undefined }, "long"],
];
for (const [dateStyle, expected] of validOptions) {
  const dtf = new Intl.DateTimeFormat("en", { dateStyle });
  const options = dtf.resolvedOptions();
  assert.sameValue(options.dateStyle, expected);
  const propdesc = Object.getOwnPropertyDescriptor(options, "dateStyle");
  if (expected === undefined) {
    assert.sameValue(propdesc, undefined);
  } else {
    assert.sameValue(propdesc.value, expected);
  }
}
