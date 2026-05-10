// Copyright 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype.getCollations
description: >
    Checks that the return value of Intl.Locale.prototype.getCollations is an Array.
info: |
  CollationsOfLocale ( loc )
  ...
  5. Return ! CreateArrayFromListAndPreferred( list, preferred ).
features: [Intl.Locale,Intl.Locale-info]
---*/

assert(Array.isArray(new Intl.Locale('en').getCollations()));
