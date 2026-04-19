// Copyright 2023 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializenumberformat
description: >
    Tests that the options minimumFractionDigits limit to the range 0 - 100.
info: |
    InitializeNumberFormat ( numberFormat, locales, options )

    25.a.ii. Set mxfd to ? DefaultNumberOption(mxfd, 0, 100, undefined).

    DefaultNumberOption ( value, minimum, maximum, fallback )

    3. If value is NaN or less than minimum or greater than maximum, throw a RangeError exception.
---*/

let wontThrow = new Intl.NumberFormat(undefined, {minimumFractionDigits: 100});

assert.throws(RangeError, function () {
        return new Intl.NumberFormat(undefined, {minimumFractionDigits: 101});
}, "Throws RangeError when minimumFractionDigits is more than 100.");
