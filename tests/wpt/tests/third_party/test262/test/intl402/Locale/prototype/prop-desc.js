// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.locale.prototype
description: >
    Checks the "prototype" property of the Locale constructor.
info: |
    Intl.Locale.prototype

    The value of Intl.Locale.prototype is %LocalePrototype%.

    This property has the attributes { [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }.
includes: [propertyHelper.js]
features: [Intl.Locale]
---*/

verifyProperty(Intl.Locale, 'prototype', {
  writable: false,
  enumerable: false,
  configurable: false,
});
