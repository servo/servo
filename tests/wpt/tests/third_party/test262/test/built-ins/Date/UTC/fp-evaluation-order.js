// Copyright (C) 2020 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.utc
description: arithmetic in Date is done on floating-point numbers
info: |
  [...]
  Return TimeClip(MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli))).

  #sec-maketime

  Let _t_ be ((_h_ `*` msPerHour `+` _m_ `*` msPerMinute) `+` _s_ `*` msPerSecond) `+` _milli_, performing the arithmetic according to IEEE 754-2019 rules (that is, as if using the ECMAScript operators `*` and `+`).

  #sec-makedate

  Return day × msPerDay + time.
---*/

assert.sameValue(Date.UTC(1970, 0, 1, 80063993375, 29, 1, -288230376151711740), 29312, 'order of operations / precision in MakeTime');
assert.sameValue(Date.UTC(1970, 0, 213503982336, 0, 0, 0, -18446744073709552000), 34447360, 'precision in MakeDate');
