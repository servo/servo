// Copyright 2012 Mozilla Corporation. All rights reserved.
// Copyright 2022 Apple Inc. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
es5id: 11.3.3
description: >
    Tests that the object returned by
    Intl.NumberFormat.prototype.resolvedOptions  has the right
    properties.
author: Norbert Lindenberg
includes: [testIntl.js, propertyHelper.js]
features: [Intl.NumberFormat-v3]
---*/

var actual = new Intl.NumberFormat().resolvedOptions();

var actual2 = new Intl.NumberFormat().resolvedOptions();
assert.notSameValue(actual2, actual, "resolvedOptions returned the same object twice.");

// this assumes the default values where the specification provides them
assert(isCanonicalizedStructurallyValidLanguageTag(actual.locale),
       "Invalid locale: " + actual.locale);
assert(isValidNumberingSystem(actual.numberingSystem),
       "Invalid numbering system: " + actual.numberingSystem);
assert.sameValue(actual.style, "decimal");
assert.sameValue(actual.minimumIntegerDigits, 1);
assert.sameValue(actual.minimumFractionDigits, 0);
assert.sameValue(actual.maximumFractionDigits, 3);
assert.sameValue(actual.useGrouping, "auto");

var dataPropertyDesc = { writable: true, enumerable: true, configurable: true };
verifyProperty(actual, "locale", dataPropertyDesc);
verifyProperty(actual, "numberingSystem", dataPropertyDesc);
verifyProperty(actual, "style", dataPropertyDesc);
verifyProperty(actual, "currency", undefined);
verifyProperty(actual, "currencyDisplay", undefined);
verifyProperty(actual, "minimumIntegerDigits", dataPropertyDesc);
verifyProperty(actual, "minimumFractionDigits", dataPropertyDesc);
verifyProperty(actual, "maximumFractionDigits", dataPropertyDesc);
verifyProperty(actual, "minimumSignificantDigits", undefined);
verifyProperty(actual, "maximumSignificantDigits", undefined);
verifyProperty(actual, "useGrouping", dataPropertyDesc);
