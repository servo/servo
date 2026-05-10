// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date-year-month-date-hours-minutes-seconds-ms
description: Order of input coercion
info: |
  3. If NewTarget is not undefined, then
    a. Let y be ? ToNumber(year).
    b. Let m be ? ToNumber(month).
    c. If date is supplied, let dt be ? ToNumber(date); else let dt be 1.
    d. If hours is supplied, let h be ? ToNumber(hours); else let h be 0.
    e. If minutes is supplied, let min be ? ToNumber(minutes); else let min be 0.
    f. If seconds is supplied, let s be ? ToNumber(seconds); else let s be 0.
    g. If ms is supplied, let milli be ? ToNumber(ms); else let milli be 0.
    h. If y is not NaN and 0 ≤ ToInteger(y) ≤ 99, let yr be 1900+ToInteger(y); otherwise,
      let yr be y.
    i. Let finalDate be MakeDate(MakeDay(yr, m, dt), MakeTime(h, min, s, milli)).
    j. Let O be ? OrdinaryCreateFromConstructor(NewTarget, "%DatePrototype%", « [[DateValue]] »).
    k. Set O.[[DateValue]] to TimeClip(UTC(finalDate)).
    l. Return O.
---*/

var log = '';
var year = { toString: function() { log += 'year'; return 0; } };
var month = { toString: function() { log += 'month'; return 0; } };
var date = { toString: function() { log += 'date'; return 1; } };
var hours = { toString: function() { log += 'hours'; return 0; } };
var minutes = { toString: function() { log += 'minutes'; return 0; } };
var seconds = { toString: function() { log += 'seconds'; return 0; } };
var ms = { toString: function() { log += 'ms'; return 0; } };

new Date(year, month, date, hours,minutes, seconds, ms);

assert.sameValue(log, 'yearmonthdatehoursminutessecondsms');
