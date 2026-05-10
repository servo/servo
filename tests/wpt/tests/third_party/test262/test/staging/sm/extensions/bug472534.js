// Copyright (C) 2024 Mozilla Corporation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: |
  pending
esid: pending
---*/
function monthNames () {
    return [
      /jan(uar(y)?)?/, 0,
      /feb(ruar(y)?)?/, 1,
      /m\u00e4r|mar|m\u00e4rz|maerz|march/, 2,
      /apr(il)?/, 3,
      /ma(i|y)/, 4,
      /jun(i|o|e)?/, 5,
      /jul(i|y)?/, 6,
      /aug(ust)?/, 7,
      /sep((t)?(ember))?/, 8,
      /o(c|k)t(ober)?/, 9,
      /nov(ember)?/, 10,
      /de(c|z)(ember)?/, 11
    ];
};

var actual = '';
var expected = '(jan(uar(y)?)?)|(feb(ruar(y)?)?)|(m\\u00e4r|mar|m\\u00e4rz|maerz|march)|(apr(il)?)|(ma(i|y))|(jun(i|o|e)?)|(jul(i|y)?)|(aug(ust)?)|(sep((t)?(ember))?)|(o(c|k)t(ober)?)|(nov(ember)?)|(de(c|z)(ember)?)';
var mn = monthNames();
for (var i = 0; i < mn.length; ++i) {
    if (actual)
        actual += '|';
    actual += '(' + mn[i++].source + ')';
}

assert.sameValue(actual, expected);

