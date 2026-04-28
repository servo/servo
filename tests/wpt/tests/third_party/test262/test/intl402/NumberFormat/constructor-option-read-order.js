// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: Checks the order of option read.
features: [Intl.NumberFormat-v3]
includes: [compareArray.js]
---*/

let optionKeys = [
    // Inside InitializeNumberFormat
    "localeMatcher",
    "numberingSystem",
    // Inside SetNumberFormatUnitOptions
        "style",
        "currency",
        "currencyDisplay",
        "currencySign",
        "unit",
        "unitDisplay",
    // End of SetNumberFormatUnitOptions
    // Back to InitializeNumberFormat
    "notation",
    // Inside SetNumberFormatDigitOptions
        "minimumIntegerDigits",
        "minimumFractionDigits",
        "maximumFractionDigits",
        "minimumSignificantDigits",
        "maximumSignificantDigits",
        "roundingIncrement",
        "roundingMode",
        "roundingPriority",
        "trailingZeroDisplay",
    // End of SetNumberFormatDigitOptions
    // Back to InitializeNumberFormat
    "compactDisplay",
    "useGrouping",
    "signDisplay"
];

// Use getters to track the order of reading known properties.
// TODO: Should we use a Proxy to detect *unexpected* property reads?
let reads = new Array();
let options = {};
optionKeys.forEach((key) => {
    Object.defineProperty(options, key, {
        get() {
            reads.push(key);
            return undefined;
        },
    });
});
new Intl.NumberFormat(undefined, options);
assert.compareArray(reads, optionKeys, "Intl.NumberFormat options read order");
