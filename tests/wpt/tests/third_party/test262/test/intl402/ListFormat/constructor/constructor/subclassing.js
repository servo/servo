// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Checks that ListFormat can be subclassed.
info: |
    Intl.ListFormat ( [ locales [ , options ] ] )

    2. Let listFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%ListFormatPrototype%", « [[InitializedListFormat]], [[Locale]], [[Type]], [[Style]] »).

features: [Intl.ListFormat]
---*/

class CustomListFormat extends Intl.ListFormat {
  constructor(locales, options) {
    super(locales, options);
    this.isCustom = true;
  }
}

const locale = "de";
const argument = ["foo", "bar"];

const real_lf = new Intl.ListFormat(locale);
assert.sameValue(real_lf.isCustom, undefined, "Custom property");

const custom_lf = new CustomListFormat(locale);
assert.sameValue(custom_lf.isCustom, true, "Custom property");

assert.sameValue(custom_lf.format(argument),
                 real_lf.format(argument),
                 "Direct call");

assert.sameValue(Intl.ListFormat.prototype.format.call(custom_lf, argument),
                 Intl.ListFormat.prototype.format.call(real_lf, argument),
                 "Indirect call");

assert.sameValue(Object.getPrototypeOf(custom_lf), CustomListFormat.prototype, "Prototype");
