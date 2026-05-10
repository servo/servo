// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 13.2.1_5
description: >
    Tests that Number.prototype.toLocaleString produces the same
    results as Intl.NumberFormat.
author: Norbert Lindenberg
includes: [compareArray.js]
---*/

var numbers = [0, -0, 1, -1, 5.5, 123, -123, -123.45, 123.44501, 0.001234,
    -0.00000000123, 0.00000000000000000000000000000123, 1.2, 0.0000000012344501,
    123445.01, 12344501000000000000000000000000000, -12344501000000000000000000000000000,
    Infinity, -Infinity, NaN];
var locales = [undefined, ["de"], ["th-u-nu-thai"], ["en"], ["ja-u-nu-jpanfin"], ["ar-u-nu-arab"]];
var options = [
    undefined,
    {style: "percent"},
    {style: "currency", currency: "EUR", currencyDisplay: "symbol"},
    {style: "currency", currency: "IQD", currencyDisplay: "symbol"},
    {style: "currency", currency: "KMF", currencyDisplay: "symbol"},
    {style: "currency", currency: "CLF", currencyDisplay: "symbol"},
    {useGrouping: false, minimumIntegerDigits: 3, minimumFractionDigits: 1, maximumFractionDigits: 3}
];

locales.forEach(function (locales) {
    options.forEach(function (options) {
        var referenceNumberFormat = new Intl.NumberFormat(locales, options);
        var referenceFormatted = numbers.map(referenceNumberFormat.format);

        var formatted = numbers.map(function (a) { return a.toLocaleString(locales, options); });
        assert.compareArray(formatted, referenceFormatted,
                            "(Testing with locales " + locales + "; options " +
                            (options ? JSON.stringify(options) : options) + ".)");
    });
});
