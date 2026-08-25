// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.ListFormat
description: Intl.ListFormat instance object is created from %ListFormatPrototype%.
info: |
    Intl.ListFormat ([ locales [ , options ]])

    2. Let listFormat be ? OrdinaryCreateFromConstructor(NewTarget, "%ListFormatPrototype%", « [[InitializedListFormat]], [[Locale]], [[Type]], [[Style]] »).
features: [Intl.ListFormat]
---*/

const value = new Intl.ListFormat();
assert.sameValue(
  Object.getPrototypeOf(value),
  Intl.ListFormat.prototype,
  "Object.getPrototypeOf(value) equals the value of Intl.ListFormat.prototype"
);
