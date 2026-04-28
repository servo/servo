// Copyright (C) 2017 Josh Wolfe. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.numberformat.prototype.formattoparts
description: Intl.NumberFormat.prototype.formatToParts called with no parameters
info: |
  Intl.NumberFormat.prototype.formatToParts ([ value ])

  3. If value is not provided, let value be undefined.
---*/

var nf = new Intl.NumberFormat();

const implicit = nf.formatToParts();
const explicit = nf.formatToParts(undefined);

// In most locales this is string "NaN", but there are exceptions, cf. "ليس رقم"
// in Arabic, "epäluku" in Finnish, "не число" in Russian, "son emas" in Uzbek etc.
const resultNaN = nf.format(NaN);
const result = [{ type: 'nan', value: resultNaN }];

assert(
  partsEquals(implicit, explicit),
  'formatToParts() should be equivalent to formatToParts(undefined)'
);

assert(
  partsEquals(implicit, result),
  'Both implicit and explicit calls should have the correct result'
);

function partsEquals(parts1, parts2) {
  if (parts1.length !== parts2.length) return false;
  for (var i = 0; i < parts1.length; i++) {
    var part1 = parts1[i];
    var part2 = parts2[i];
    if (part1.type !== part2.type) return false;
    if (part1.value !== part2.value) return false;
  }
  return true;
}
