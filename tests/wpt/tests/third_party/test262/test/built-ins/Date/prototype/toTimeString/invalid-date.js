// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.totimestring
description: Invalid Dates are rendered as "Invalid Date"
info: |
  Date.prototype.toTimeString ( )

  ...
  3. If tv is NaN, return "Invalid Date".
  ...
---*/

assert.sameValue(new Date(NaN).toTimeString(), "Invalid Date");
