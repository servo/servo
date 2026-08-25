// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.resolvedoptions
description: Verifies the property order for the object returned by resolvedOptions().
features: [Intl.NumberFormat-unified]
---*/

const options = new Intl.NumberFormat([], {
  "style": "currency",
  "currency": "EUR",
  "currencyDisplay": "symbol",
  "minimumSignificantDigits": 1,
  "maximumSignificantDigits": 2,
}).resolvedOptions();

const expected = [
  "locale",
  "numberingSystem",
  "style",
  "currency",
  "currencyDisplay",
  "currencySign",
  "minimumIntegerDigits",
  "minimumSignificantDigits",
  "maximumSignificantDigits",
  "useGrouping",
  "notation",
  "signDisplay",
];

const actual = Object.getOwnPropertyNames(options);

// Ensure all expected items are in actual and also allow other properties
// implemented in new proposals.
assert(actual.indexOf("locale") > -1, "\"locale\" is present");
for (var i = 1; i < expected.length; i++) {
  // Ensure the order as expected but allow additional new property in between
  assert(actual.indexOf(expected[i-1]) < actual.indexOf(expected[i]), `"${expected[i-1]}" precedes "${expected[i]}"`);
}
