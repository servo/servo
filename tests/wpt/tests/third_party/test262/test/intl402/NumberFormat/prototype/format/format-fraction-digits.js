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
    "0": "000.0",
    "-0": "-000.0",
    "123": "123.0",
    "-123": "-123.0",
    "12345": "12345.0",
    "-12345": "-12345.0",
    "123.45": "123.45",
    "-123.45": "-123.45",
    "123.444499": "123.444",
    "-123.444499": "-123.444",
    "123.444500": "123.445",
    "-123.444500": "-123.445",
    "123.44501": "123.445",
    "-123.44501": "-123.445",
    "0.001234": "000.001",
    "-0.001234": "-000.001",
    "0.00000000123": "000.0",
    "-0.00000000123": "-000.0",
    "0.00000000000000000000000000000123": "000.0",
    "-0.00000000000000000000000000000123": "-000.0",
    "1.2": "001.2",
    "-1.2": "-001.2",
    "0.0000000012344501": "000.0",
    "-0.0000000012344501": "-000.0",
    "123445.01": "123445.01",
    "-123445.01": "-123445.01",
};

testNumberFormat(locales, numberingSystems,
    {useGrouping: false, minimumIntegerDigits: 3, minimumFractionDigits: 1, maximumFractionDigits: 3},
    testData);
