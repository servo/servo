// Copyright (C) 2017 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-todatestring
description: Invalid Dates are rendered as "Invalid Date"
info: |
  ToDateString ( tv )

  ...
  2. If tv is NaN, return "Invalid Date".
  ...
---*/

assert.sameValue(new Date(NaN).toString(), "Invalid Date");
