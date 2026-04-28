// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.todatestring
description: Test the format of the date from toDateString
info: |
  Date.prototype.toDateString ( )

  5. Return DateString(_t_).
---*/

let dateRegExp = /^(Sun|Mon|Tue|Wed|Thu|Fri|Sat) (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [0-9]{2} [0-9]{4}$/
let match = dateRegExp.exec(new Date(0).toDateString());
assert.notSameValue(null, match);

// Years are padded to the left with zeroes
match = dateRegExp.exec(new Date('0020-01-01T00:00:00Z').toDateString());
assert.notSameValue(null, match);
