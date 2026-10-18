// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: >
  Language subtags with more than three letters are supported.
info: |
  TextDirectionOfLocale ( loc )
  ...
  2. Let script be GetLocaleScript(locale).
  3. If script is undefined, then
    a. Let maximal be the result of the Add Likely Subtags algorithm applied to locale. If an error is signaled, return undefined.
    b. Set script to GetLocaleScript(maximal).
    c. If script is undefined, return undefined.
  4. If the default general ordering of characters within a line in script is right-to-left, return "rtl".
  5. If the default general ordering of characters within a line in script is left-to-right, return "ltr".
  6. Return undefined.

  NOTE 1:
  When the direction of default general ordering of characters within a line in
  the script cannot be determined, or the direction is neither right-to-left nor
  left-to-right, then undefined will be returned.
features: [Intl.Locale, Intl.Locale-info]
---*/

// "abcdefgh" is not a registered language, so it's not possible to infer its script
// through adding likely subtags.
assert.sameValue(
  new Intl.Locale("abcdefgh").getTextInfo().direction,
  undefined,
  `with locale "abcdefgh"`
);

// Script subtag is present, "Latn" is written left-to-right.
assert.sameValue(
  new Intl.Locale("abcdefgh-Latn").getTextInfo().direction,
  "ltr",
  `with locale "abcdefgh-Latn"`
);

// Script subtag is present, "Arab" is written right-to-left.
assert.sameValue(
  new Intl.Locale("abcdefgh-Arab").getTextInfo().direction,
  "rtl",
  `with locale "abcdefgh-Arab"`
);

// Script subtag is present, "Zzzz" is neither written left-to-right nor right-to-left.
assert.sameValue(
  new Intl.Locale("abcdefgh-Zzzz").getTextInfo().direction,
  undefined,
  `with locale "abcdefgh-Zzzz"`
);
