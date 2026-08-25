// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.subtract
description: Fallback value for overflow option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-totemporaloverflow step 1:
      1. Return ? GetOption(_normalizedOptions_, *"overflow"*, « String », « *"constrain"*, *"reject"* », *"constrain"*).
    sec-temporal.calendar.prototype.dateadd step 7:
      7. Let _overflow_ be ? ToTemporalOverflow(_options_).
    sec-temporal-adddatetime step 4:
      4. Let _addedDate_ be ? CalendarDateAdd(_calendar_, _datePart_, _dateDuration_, _options_).
    sec-temporal.plaindatetime.prototype.subtract step 5:
      5. Let _result_ be ? AddDateTime(_dateTime_.[[ISOYear]], _dateTime_.[[ISOMonth]], _dateTime_.[[ISODay]], _dateTime_.[[ISOHour]], _dateTime_.[[ISOMinute]], _dateTime_.[[ISOSecond]], _dateTime_.[[ISOMillisecond]], _dateTime_.[[ISOMicrosecond]], _dateTime_.[[ISONanosecond]], _dateTime_.[[Calendar]], −_duration_.[[Years]], −_duration_.[[Months]], −_duration_.[[Weeks]], −_duration_.[[Days]], −_duration_.[[Hours]], −_duration_.[[Minutes]], −_duration_.[[Seconds]], −_duration_.[[Milliseconds]], −_duration_.[[Microseconds]], −_duration_.[[Nanoseconds]], _options_).
includes: [temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.PlainDateTime(2000, 5, 31, 12);
const duration = new Temporal.Duration(3, 1);

const explicit = datetime.subtract(duration, { overflow: undefined });
TemporalHelpers.assertPlainDateTime(explicit, 1997, 4, "M04", 30, 12, 0, 0, 0, 0, 0, "default overflow is constrain");
const implicit = datetime.subtract(duration, {});
TemporalHelpers.assertPlainDateTime(implicit, 1997, 4, "M04", 30, 12, 0, 0, 0, 0, 0, "default overflow is constrain");
const lambda = datetime.subtract(duration, () => {});
TemporalHelpers.assertPlainDateTime(lambda, 1997, 4, "M04", 30, 12, 0, 0, 0, 0, 0, "default overflow is constrain");
