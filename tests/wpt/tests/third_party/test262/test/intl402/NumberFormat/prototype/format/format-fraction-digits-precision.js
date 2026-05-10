// Copyright 2011-2012 Norbert Lindenberg. All rights reserved.
// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.3.2_TRF
description: >
    Tests that the digits are determined correctly when specifying
    pre/post decimal digits.
author: Norbert Lindenberg
includes: [testIntl.js]
---*/

var locales = [
    new Intl.NumberFormat().resolvedOptions().locale,
    "ar", "de", "th", "ja"
];
var numberingSystems = [
    "arab",
    "latn",
    "thai",
    "hanidec"
];
var testData = {
    // Ref tc39/ecma402#128
    "12344501000000000000000000000000000": "12344501000000000000000000000000000.0",
    "-12344501000000000000000000000000000": "-12344501000000000000000000000000000.0"
};

testNumberFormat(locales, numberingSystems,
    {useGrouping: false, minimumIntegerDigits: 3, minimumFractionDigits: 1, maximumFractionDigits: 3},
    testData);
