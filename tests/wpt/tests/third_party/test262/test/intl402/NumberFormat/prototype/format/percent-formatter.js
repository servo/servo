// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.3.2_FN_3_b
description: >
    Tests that Intl.NumberFormat.prototype.format  formats percent
    values properly.
author: Roozbeh Pournader
---*/

var numberFormatter = new Intl.NumberFormat();
var percentFormatter = new Intl.NumberFormat(undefined, {style: 'percent'});

var formattedTwenty = numberFormatter.format(20);
var formattedTwentyPercent = percentFormatter.format(0.20);

// FIXME: May not work for some theoretical locales where percents and
// normal numbers are formatted using different numbering systems.
assert.notSameValue(formattedTwentyPercent.indexOf(formattedTwenty), -1, "Intl.NumberFormat's formatting of 20% does not include a formatting of 20 as a substring.");

// FIXME: Move this to somewhere appropriate
assert.notSameValue(percentFormatter.format(0.011), percentFormatter.format(0.02), 'Intl.NumberFormat is formatting 1.1% and 2% the same way.');
