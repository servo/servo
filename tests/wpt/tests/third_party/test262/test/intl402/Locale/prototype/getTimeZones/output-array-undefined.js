// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getTimeZones
description: >
    Checks that the return value of Intl.Locale.prototype.timeZones is undefined
    when no region subtag is used.
info: |
  get Intl.Locale.prototype.timeZones
  ...
  4. If the unicode_language_id production of locale does not contain the
  ["-" unicode_region_subtag] sequence, return undefined.
features: [Intl.Locale,Intl.Locale-info]
---*/

assert.sameValue(new Intl.Locale('en').getTimeZones(), undefined);
