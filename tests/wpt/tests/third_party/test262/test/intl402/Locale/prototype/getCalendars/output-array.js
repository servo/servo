// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getCalendars
description: >
    Checks that the return value of Intl.Locale.prototype.getCalendars is an Array.
info: |
  CalendarsOfLocale ( loc )
  ...
  5. Return ! CreateArrayFromListAndPreferred( list, preferred ).
features: [Intl.Locale,Intl.Locale-info]
---*/

assert(Array.isArray(new Intl.Locale('en').getCalendars()));
assert(new Intl.Locale('en').getCalendars().length > 0, 'array has at least one element');
