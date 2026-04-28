// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getTimeZones
description: >
    Checks that the return value of Intl.Locale.prototype.getTimeZones is an Array
    when a region subtag is used.
info: |
  TimeZonesOfLocale ( loc )
  ...
  5. Return ! CreateArrayFromListAndPreferred( list, preferred ).
features: [Intl.Locale,Intl.Locale-info]
---*/

assert(Array.isArray(new Intl.Locale('en-US').getTimeZones()));
assert(new Intl.Locale('en-US').getTimeZones().length > 0, 'array has at least one element');
