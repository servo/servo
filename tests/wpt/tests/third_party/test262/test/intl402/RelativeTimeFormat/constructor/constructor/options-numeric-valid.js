// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks handling of valid values for the numeric option to the RelativeTimeFormat constructor.
info: |
    InitializeRelativeTimeFormat (relativeTimeFormat, locales, options)
    16. Let numeric be ? GetOption(options, "numeric", "string", «"always", "auto"», "always").
    17. Set relativeTimeFormat.[[Numeric]] to numeric.
features: [Intl.RelativeTimeFormat]
---*/

const validOptions = [
  [undefined, "always"],
  ["always", "always"],
  ["auto", "auto"],
  [{ toString() { return "auto"; } }, "auto"],
];

for (const [validOption, expected] of validOptions) {
  const tf = new Intl.RelativeTimeFormat([], {"numeric": validOption});
  const resolvedOptions = tf.resolvedOptions();
  assert.sameValue(resolvedOptions.numeric, expected);
}
