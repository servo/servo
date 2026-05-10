// Copyright (C) 2015 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-timeclip
description: TimeClip converts negative zero to positive zero
info: |
  20.3.1.15 TimeClip (time)

  ...
  3. Return ToInteger(time) + (+0). (Adding a positive zero converts -0 to +0.)
es6id: 20.3.1.15
---*/

var date = new Date(-0);

assert.sameValue(date.getTime(), +0, "TimeClip does not return negative zero");
