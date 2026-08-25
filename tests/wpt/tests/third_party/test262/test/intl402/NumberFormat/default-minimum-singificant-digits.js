// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Tests that the default value of minimumSignificantDigits is 1.
esid: sec-setnfdigitoptions
---*/

// maximumSignificantDigits needs to be in range from minimumSignificantDigits
// to 21 (both inclusive). Setting maximumSignificantDigits to 0 will throw a
// RangeError if the (default) minimumSignificantDigits is at least 1.
assert.throws(RangeError, function() {
  Intl.NumberFormat(undefined, {maximumSignificantDigits: 0});
});

// If nothing is thrown, check that the options are resolved appropriately.
var res = Intl.NumberFormat(undefined, {maximumSignificantDigits: 1})

assert.sameValue(Object.getPrototypeOf(res), Intl.NumberFormat.prototype, 'result is an instance of NumberFormat')
assert.sameValue(res.resolvedOptions().minimumSignificantDigits, 1, 'default minimumSignificantDigits')
assert.sameValue(res.resolvedOptions().maximumSignificantDigits, 1, 'sets maximumSignificantDigits')
