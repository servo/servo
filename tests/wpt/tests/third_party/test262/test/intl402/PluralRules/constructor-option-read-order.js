// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.pluralrules
description: Checks the order of options read.
features: [Intl.NumberFormat-v3]
includes: [compareArray.js, temporalHelpers.js]
---*/

let optionKeys = [
    // Inside new PluralRules()
    "get options.localeMatcher",
    "get options.type",
    "get options.notation",
    "get options.compactDisplay",
    // Inside SetNumberFormatDigitOptions
        "get options.minimumIntegerDigits",
        "get options.minimumFractionDigits",
        "get options.maximumFractionDigits",
        "get options.minimumSignificantDigits",
        "get options.maximumSignificantDigits",
        "get options.roundingIncrement",
        "get options.roundingMode",
        "get options.roundingPriority",
        "get options.trailingZeroDisplay",
    // End of SetNumberFormatDigitOptions
];

const reads = [];
const options = TemporalHelpers.propertyBagObserver(reads, {}, "options");
new Intl.PluralRules(undefined, options);
assert.compareArray(reads, optionKeys, "Intl.PluralRules options read order");
