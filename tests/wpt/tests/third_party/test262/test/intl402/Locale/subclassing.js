// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Locale
description: Checks that Locale can be subclassed.
info: |
    Intl.Locale( tag [, options] )

    6. Let locale be ? OrdinaryCreateFromConstructor(NewTarget, %LocalePrototype%, internalSlotsList).

features: [Intl.Locale]
---*/

class CustomLocale extends Intl.Locale {
  constructor(locales, options) {
    super(locales, options);
    this.isCustom = true;
  }
}

var locale = new CustomLocale("de");
assert.sameValue(locale.isCustom, true, "Custom property");
assert.sameValue(locale.toString(), "de", "Direct call");
assert.sameValue(Intl.Locale.prototype.toString.call(locale), "de", "Indirect call");
assert.sameValue(Object.getPrototypeOf(locale), CustomLocale.prototype, "Prototype");
