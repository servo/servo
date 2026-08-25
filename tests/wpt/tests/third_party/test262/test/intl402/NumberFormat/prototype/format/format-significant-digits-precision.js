// Copyright 2011-2012 Norbert Lindenberg. All rights reserved.
// Copyright 2012 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.3.2_TRP
description: >
    Tests that the digits are determined correctly when specifying
    significant digits.
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
    "123.44500": "123.45",
    "-123.44500": "-123.45",
};

testNumberFormat(locales, numberingSystems,
    {useGrouping: false, minimumSignificantDigits: 3, maximumSignificantDigits: 5},
    testData);
