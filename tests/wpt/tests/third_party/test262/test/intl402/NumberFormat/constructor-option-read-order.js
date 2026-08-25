// Copyright 2023 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: Checks the order of options read.
features: [Intl.NumberFormat-unified, Intl.NumberFormat-v3]
includes: [compareArray.js, temporalHelpers.js]
---*/

let optionKeys = [
    // Inside new NumberFormat()
    "get options.localeMatcher",
    "get options.numberingSystem",
    // Inside SetNumberFormatUnitOptions
        "get options.style",
        "get options.currency",
        "get options.currencyDisplay",
        "get options.currencySign",
        "get options.unit",
        "get options.unitDisplay",
    // End of SetNumberFormatUnitOptions
    // Back to new NumberFormat()
    "get options.notation",
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
    // Back to new NumberFormat()
    "get options.compactDisplay",
    "get options.useGrouping",
    "get options.signDisplay"
];

let reads = [];
let options = TemporalHelpers.propertyBagObserver(reads, {}, "options");
new Intl.NumberFormat(undefined, options);
assert.compareArray(reads, optionKeys, "Intl.NumberFormat options read order");
