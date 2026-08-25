// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.RelativeTimeFormat
description: >
    Intl.RelativeTimeFormat instance object is created from %RelativeTimeFormatPrototype%.
info: |
    Intl.RelativeTimeFormat ([ locales [ , options ]])

    2. Let relativeTimeFormat be ! OrdinaryCreateFromConstructor(NewTarget, "%RelativeTimeFormatPrototype%").
features: [Intl.RelativeTimeFormat]
---*/

const value = new Intl.RelativeTimeFormat();
assert.sameValue(
  Object.getPrototypeOf(value),
  Intl.RelativeTimeFormat.prototype,
  "Object.getPrototypeOf(value) equals the value of Intl.RelativeTimeFormat.prototype"
);
