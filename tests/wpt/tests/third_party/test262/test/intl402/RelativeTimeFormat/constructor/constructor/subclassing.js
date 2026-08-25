// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: Checks that RelativeTimeFormat can be subclassed.
info: |
    Intl.RelativeTimeFormat ( [ locales [ , options ] ] )

    2. Let relativeTimeFormat be ! OrdinaryCreateFromConstructor(NewTarget, "%RelativeTimeFormatPrototype%", « [[InitializedRelativeTimeFormat]] »).

features: [Intl.RelativeTimeFormat]
---*/

class CustomRelativeTimeFormat extends Intl.RelativeTimeFormat {
  constructor(locales, options) {
    super(locales, options);
    this.isCustom = true;
  }
}

const locale = "de";
const value = 7;
const unit = "day";

const real_rtf = new Intl.RelativeTimeFormat(locale);
assert.sameValue(real_rtf.isCustom, undefined, "Custom property");

const custom_rtf = new CustomRelativeTimeFormat(locale);
assert.sameValue(custom_rtf.isCustom, true, "Custom property");

assert.sameValue(custom_rtf.format(value, unit),
                 real_rtf.format(value, unit),
                 "Direct call");

assert.sameValue(Intl.RelativeTimeFormat.prototype.format.call(custom_rtf, value, unit),
                 Intl.RelativeTimeFormat.prototype.format.call(real_rtf, value, unit),
                 "Indirect call");

assert.sameValue(Object.getPrototypeOf(custom_rtf), CustomRelativeTimeFormat.prototype, "Prototype");
