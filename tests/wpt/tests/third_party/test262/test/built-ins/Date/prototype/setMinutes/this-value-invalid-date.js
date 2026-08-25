// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-date.prototype.setminutes
description: >
  Behavior when the "this" value is a Date object describing an invald date
info: |
  1. Let t be LocalTime(? thisTimeValue(this value)).
  2. Let h be ? ToNumber(hour).
  3. If min is not specified, let m be MinFromTime(t); otherwise, let m be ?
     ToNumber(min).
  4. If sec is not specified, let s be SecFromTime(t); otherwise, let s be ?
     ToNumber(sec).
  5. If ms is not specified, let milli be msFromTime(t); otherwise, let milli
     be ? ToNumber(ms).
  6. Let date be MakeDate(Day(t), MakeTime(h, m, s, milli)).
  7. Let u be TimeClip(UTC(date)).
  8. Set the [[DateValue]] internal slot of this Date object to u.
  9. Return u.
---*/

var date = new Date(NaN);
var result;

result = date.setMinutes(0);

assert.sameValue(result, NaN, 'return value (hour)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (hour)');

result = date.setMinutes(0, 0);

assert.sameValue(result, NaN, 'return value (minute)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (minute)');

result = date.setMinutes(0, 0, 0);

assert.sameValue(result, NaN, 'return value (second)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (second)');

result = date.setMinutes(0, 0, 0, 0);

assert.sameValue(result, NaN, 'return value (ms)');
assert.sameValue(date.getTime(), NaN, '[[DateValue]] internal slot (ms)');
