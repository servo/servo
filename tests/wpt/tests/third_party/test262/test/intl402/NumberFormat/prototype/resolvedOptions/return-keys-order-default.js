// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-intl.numberformat.prototype.resolvedoptions
description: order of property keys for the object returned by resolvedOptions()
features: [Intl.NumberFormat-v3]
includes: [compareArray.js]
---*/

const allKeys = [
    'locale',
    'numberingSystem',
    'style',
    'currency',
    'currencyDisplay',
    'currencySign',
    'unit',
    'unitDisplay',
    'minimumIntegerDigits',
    'minimumFractionDigits',
    'maximumFractionDigits',
    'minimumSignificantDigits',
    'maximumSignificantDigits',
    'useGrouping',
    'notation',
    'compactDisplay',
    'signDisplay',
    'roundingIncrement',
    'roundingMode',
    'roundingPriority',
    'trailingZeroDisplay'
];

const optionsBase = { notation: 'compact' };
const optionsExtensions = [
    { style: 'currency', currency: 'XTS' },
    { style: 'unit', unit: 'percent' },
];
optionsExtensions.forEach((optionsExtension) => {
    const options = Object.assign({}, optionsBase, optionsExtension);
    const nf = new Intl.NumberFormat(undefined, options);
    const resolved = nf.resolvedOptions();
    const resolvedKeys = Reflect.ownKeys(resolved);
    const expectedKeys = allKeys.filter(key => key in resolved);
    assert.compareArray(resolvedKeys, expectedKeys,
        'resolvedOptions() property key order with options ' + JSON.stringify(options));
});
