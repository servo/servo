// Copyright 2012 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 11.3.2_FN_1
description: >
    Tests that Intl.NumberFormat.prototype.format  doesn't treat all
    numbers as negative.
info: |
    PartitionNumberPattern ( numberFormat, x )
    1. If x is not NaN and x < 0 or _x_ is -0, then
       a. Let _x_ be -_x_.
       b. Let _pattern_ be _numberFormat_.[[NegativePattern]].
author: Roozbeh Pournader
---*/

var formatter = new Intl.NumberFormat();
  
assert.notSameValue(formatter.format(1), formatter.format(-1), 'Intl.NumberFormat is formatting 1 and -1 the same way.');

assert.notSameValue(formatter.format(0), formatter.format(-0), 'Intl.NumberFormat is formatting 0 and -0 the same way.');
