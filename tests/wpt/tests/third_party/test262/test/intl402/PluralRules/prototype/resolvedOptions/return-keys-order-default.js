// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.pluralrules.prototype.resolvedoptions
description: order of property keys for the object returned by resolvedOptions()
features: [Intl.NumberFormat-v3]
includes: [compareArray.js]
---*/

const allKeys = [
    'locale',
    'type',
    'notation',
    'minimumIntegerDigits',
    'minimumFractionDigits',
    'maximumFractionDigits',
    'minimumSignificantDigits',
    'maximumSignificantDigits',
    'pluralCategories',
    'roundingIncrement',
    'roundingMode',
    'roundingPriority',
    'trailingZeroDisplay'
];

const options = [
    { },
    { minimumSignificantDigits: 3 },
    { minimumFractionDigits: 3 },
];
options.forEach((option) => {
    const nf = new Intl.PluralRules(undefined, option);
    const resolved = nf.resolvedOptions();
    const resolvedKeys = Reflect.ownKeys(resolved);
    const expectedKeys = allKeys.filter(key => key in resolved);
    assert.compareArray(resolvedKeys, expectedKeys,
        'resolvedOptions() property key order with options ' + JSON.stringify(options));
});
