// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializepluralrules
description: Checks the order of option read.
features: [Intl.NumberFormat-v3]
includes: [compareArray.js]
---*/

let optionKeys = [
    // Inside InitializePluralRules
    "localeMatcher",
    "type",
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
new Intl.PluralRules(undefined, options);
assert.compareArray(reads, optionKeys, "Intl.PluralRules options read order");
