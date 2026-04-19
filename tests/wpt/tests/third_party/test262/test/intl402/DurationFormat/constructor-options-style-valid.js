// Copyright 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat
description: Checks handling of valid values for the style option to the DurationFormat constructor.
info: |
    InitializeDurationFormat (DurationFormat, locales, options)
    (...)
    13. Let style be ? GetOption(options, "style", "string", « "long", "short", "narrow", "digital" », "short").
    14. Set durationFormat.[[Style]] to style.
features: [Intl.DurationFormat]
---*/

const validOptions = [
  [undefined, "short"],
  ["long", "long"],
  ["short", "short"],
  ["narrow", "narrow"],
  ["digital", "digital"],
  [{ toString() { return "short"; } }, "short"],
  [{ toString() { return "long"; } }, "long"],
  [{ toString() { return "narrow"; } }, "narrow"],
  [{ toString() { return "digital"; } }, "digital"],
];

for (const [validOption, expected] of validOptions) {
  const df = new Intl.DurationFormat([], {"style": validOption});
  const resolvedOptions = df.resolvedOptions();
  assert.sameValue(resolvedOptions.style, expected);
}
