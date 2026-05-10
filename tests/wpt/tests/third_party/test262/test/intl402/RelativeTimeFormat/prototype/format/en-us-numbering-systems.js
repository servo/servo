// Copyright 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat.prototype.format
description: Checks that numberingSystem option used correctly.
info: |
    17.5.2 PartitionRelativeTimePattern ( relativeTimeFormat, value, unit )

    11. Let fv be PartitionNumberPattern(relativeTimeFormat.[[NumberFormat]], ℝ(value)).

locale: [en-US]
---*/

const localesAndResults = [
  ["en-US"],
  ["en-US-u-nu-arab"],
  ["en-US-u-nu-deva"],
  ["en-US-u-nu-hanidec"],
];
const seconds = 1234567890;

for (const locale of localesAndResults){
  const formatted = new Intl.RelativeTimeFormat(locale, {style: "short"}).format(seconds, "seconds");
  const expected = new Intl.NumberFormat(locale).format(seconds);
  assert.sameValue(formatted.includes(expected), true, `locale: ${locale}`);
}
