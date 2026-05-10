// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.totimestring
description: Test the format of the time from toTimeString
info: |
  Date.prototype.toTimeString ( )

  5. Return the String value formed by concatenating TimeString(_t_) and TimeZoneString(_tv_).
---*/

let timeRegExp = /^[0-9]{2}:[0-9]{2}:[0-9]{2} GMT[+-][0-9]{4}( \(.+\))?$/
let match = timeRegExp.exec(new Date(0).toTimeString());
assert.notSameValue(null, match);
