// Copyright 2016 Mozilla Corporation. All rights reserved.
// This code is governed by the license found in the LICENSE file.

/*---
esid: sec-Intl.PluralRules.prototype.resolvedOptions
description: >
    Tests that the object returned by
    Intl.PluralRules.prototype.resolvedOptions  has the right
    properties.
author: Zibi Braniecki
includes: [testIntl.js, propertyHelper.js]
---*/

var actual = new Intl.PluralRules().resolvedOptions();

var actual2 = new Intl.PluralRules().resolvedOptions();
assert.notSameValue(actual2, actual, "resolvedOptions returned the same object twice.");

// this assumes the default values where the specification provides them
assert(isCanonicalizedStructurallyValidLanguageTag(actual.locale),
       "Invalid locale: " + actual.locale);
assert.sameValue(actual.type, "cardinal");
assert.sameValue(actual.notation, "standard");
assert.sameValue(actual.minimumIntegerDigits, 1);
assert.sameValue(actual.minimumFractionDigits, 0);
assert.sameValue(actual.maximumFractionDigits, 3);

var dataPropertyDesc = { writable: true, enumerable: true, configurable: true };
verifyProperty(actual, "locale", dataPropertyDesc);
verifyProperty(actual, "type", dataPropertyDesc);
verifyProperty(actual, "notation", dataPropertyDesc);
verifyProperty(actual, "currency", undefined);
verifyProperty(actual, "currencyDisplay", undefined);
verifyProperty(actual, "minimumIntegerDigits", dataPropertyDesc);
verifyProperty(actual, "minimumFractionDigits", dataPropertyDesc);
verifyProperty(actual, "maximumFractionDigits", dataPropertyDesc);
verifyProperty(actual, "minimumSignificantDigits", undefined);
verifyProperty(actual, "maximumSignificantDigits", undefined);
