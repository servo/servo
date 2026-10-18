// Copyright 2026 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale.prototype.getTextInfo
description: >
  Don't return "ltr" direction for scripts which aren't left-to-right.
info: |
  TextDirectionOfLocale ( loc )
  ...
  4. If the default general ordering of characters within a line in script is right-to-left, return "rtl".
  5. If the default general ordering of characters within a line in script is left-to-right, return "ltr".
  6. Return undefined.

  NOTE 1:
  When the direction of default general ordering of characters within a line in
  the script cannot be determined, or the direction is neither right-to-left nor
  left-to-right, then undefined will be returned.
features: [Intl.Locale, Intl.Locale-info]
---*/

// Arabic, spoken-only content.
var arDir = new Intl.Locale("ar-Zxxx").getTextInfo().direction;
// Allow absent data, returning undefined, but reject "ltr".
assert(arDir === "rtl" || arDir === undefined, "with locale 'ar-Zxxx'");

// Urdu, with script Arabic (Nastaliq variant).
var urDir = new Intl.Locale("ur-Aran").getTextInfo().direction;
// Allow absent data, returning undefined, but reject "ltr".
assert(urDir === "rtl" || urDir === undefined, "with locale 'ur-Aran'");
