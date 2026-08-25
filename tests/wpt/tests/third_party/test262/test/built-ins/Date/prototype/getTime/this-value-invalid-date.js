// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.gettime
description: Return value for invalid date
info: |
  1. Return ? thisTimeValue(this value). 
---*/

assert.sameValue(new Date(NaN).getTime(), NaN);
