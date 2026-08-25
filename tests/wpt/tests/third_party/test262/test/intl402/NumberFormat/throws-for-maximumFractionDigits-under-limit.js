// Copyright 2023 Google Inc. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat
description: >
    Tests that the options maximumFractionDigits limit to the range 0 - 100.
info: |
  SetNumberFormatDigitOptions ( intlObj, options, mnfdDefault, mxfdDefault, notation )

  23.a.ii. Set _mxfd_ to ? DefaultNumberOption(_mxfd_, 0, 100, *undefined*).

  DefaultNumberOption ( value, minimum, maximum, fallback )

  3. If _value_ is not finite or ℝ(_value_) < _minimum_ or ℝ(_value_) >
     _maximum_, throw a *RangeError* exception.
---*/

let wontThrow = new Intl.NumberFormat(undefined, {maximumFractionDigits: 0});

assert.throws(RangeError, function () {
        return new Intl.NumberFormat(undefined, {maximumFractionDigits: -1});
}, "Throws RangeError when maximumFractionDigits is less than 0.");
