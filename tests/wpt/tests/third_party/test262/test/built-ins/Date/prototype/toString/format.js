// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.tostring
description: Test the format of the time from toString
info: |
  Runtime Semantics: ToDateString( _tv_ )

  4. Return the String value formed by concatenating DateString(_t_), `" "`, TimeString(_t_), and TimeZoneString(_tv_).

---*/

let stringRegExp = /^(Sun|Mon|Tue|Wed|Thu|Fri|Sat) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [0-9]{2} [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT[+-][0-9]{4}( \(.+\))?$/
let match = stringRegExp.exec(new Date(0).toString());
assert.notSameValue(null, match);

// Years are padded to the left with zeroes
match = stringRegExp.exec(new Date('0020-01-01T00:00:00Z').toString());
assert.notSameValue(null, match);
