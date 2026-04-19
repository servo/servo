// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toutcstring
description: Test the format of the date from toUTCString
info: |
  Date.prototype.toUTCString ( )

  4. Return the String value formed by concatenating DateString(_tv_, `", "`), `" "`, and TimeString(_tv_).
---*/

let utcRegExp = /^(Sun|Mon|Tue|Wed|Thu|Fri|Sat), [0-9]{2} (Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec) [0-9]{4} [0-9]{2}:[0-9]{2}:[0-9]{2} GMT$/
let match = utcRegExp.exec(new Date(0).toUTCString());
assert.notSameValue(null, match);

// Years are padded to the left with zeroes
assert.sameValue('Wed, 01 Jan 0020 00:00:00 GMT', new Date('0020-01-01T00:00:00Z').toUTCString());
