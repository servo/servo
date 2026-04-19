// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks handling of valid values for the style option to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    14. Let s be ? GetOption(options, "style", "string", «"long", "short", "narrow"», "long").
    15. Set relativeTimeFormat.[[Style]] to s.
features: [Intl.RelativeTimeFormat]
---*/

const validOptions = [
  [undefined, "long"],
  ["long", "long"],
  ["short", "short"],
  ["narrow", "narrow"],
  [{ toString() { return "narrow"; } }, "narrow"],
];

for (const [validOption, expected] of validOptions) {
  const tf = new Intl.RelativeTimeFormat([], {"style": validOption});
  const resolvedOptions = tf.resolvedOptions();
  assert.sameValue(resolvedOptions.style, expected);
}
