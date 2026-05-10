// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal
description: Test June 2024 Temporal API removals
info: >
  This staging test fails if the Temporal API removals, which reached consensus
  in the TC39 meeting of June 2024, are not implemented.

  Technically, it's spec-compliant to expose extra properties and methods, as
  long as they are not in the forbidden extensions list. So it's possible that
  your implementation might fail this test while still being compliant.

  But still, please don't do that! If you believe this test is in error, open an
  issue on the Temporal proposal repo: https://github.com/tc39/proposal-temporal
features: [Temporal]
---*/

assert(!("Calendar" in Temporal), "Temporal.Calendar should not exist");
assert(!("TimeZone" in Temporal), "Temporal.TimeZone should not exist");

const { Instant } = Temporal;
assert(!("fromEpochMicroseconds" in Instant), "Temporal.Instant.fromEpochMicroseconds should not exist");
assert(!("fromEpochSeconds" in Instant), "Temporal.Instant.fromEpochSeconds should not exist");

const InstantProto = Temporal.Instant.prototype;
assert(!("epochMicroseconds" in InstantProto), "Temporal.Instant.prototype.epochMicroseconds should not exist");
assert(!("epochSeconds" in InstantProto), "Temporal.Instant.prototype.epochSeconds should not exist");
assert(!("toZonedDateTime" in InstantProto), "Temporal.Instant.prototype.toZonedDateTime should not exist");

const { Now } = Temporal;
assert(!("plainDate" in Now), "Temporal.Now.plainDate should not exist");
assert(!("plainDateTime" in Now), "Temporal.Now.plainDateTime should not exist");
assert(!("zonedDateTime" in Now), "Temporal.Now.zonedDateTime should not exist");

const PlainDateProto = Temporal.PlainDate.prototype;
assert(!("getCalendar" in PlainDateProto), "Temporal.PlainDate.prototype.getCalendar should not exist");
assert(!("getISOFields" in PlainDateProto), "Temporal.PlainDate.prototype.getISOFields should not exist");

const PlainDateTimeProto = Temporal.PlainDateTime.prototype;
assert(!("getCalendar" in PlainDateTimeProto), "Temporal.PlainDateTime.prototype.getCalendar should not exist");
assert(!("getISOFields" in PlainDateTimeProto), "Temporal.PlainDateTime.prototype.getISOFields should not exist");
assert(!("toPlainMonthDay" in PlainDateTimeProto), "Temporal.PlainDateTime.prototype.toPlainMonthDay should not exist");
assert(!("toPlainYearMonth" in PlainDateTimeProto), "Temporal.PlainDateTime.prototype.toPlainYearMonth should not exist");
assert(!("withPlainDate" in PlainDateTimeProto), "Temporal.PlainDateTime.prototype.withPlainDate should not exist");

const PlainMonthDayProto = Temporal.PlainMonthDay.prototype;
assert(!("getCalendar" in PlainMonthDayProto), "Temporal.PlainMonthDay.prototype.getCalendar should not exist");
assert(!("getISOFields" in PlainMonthDayProto), "Temporal.PlainMonthDay.prototype.getISOFields should not exist");

const PlainTimeProto = Temporal.PlainTime.prototype;
assert(!("getISOFields" in PlainTimeProto), "Temporal.PlainTime.prototype.getISOFields should not exist");
assert(!("toPlainDateTime" in PlainTimeProto), "Temporal.PlainTime.prototype.toPlainDateTime should not exist");
assert(!("toZonedDateTime" in PlainTimeProto), "Temporal.PlainTime.prototype.toZonedDateTime should not exist");

const PlainYearMonthProto = Temporal.PlainYearMonth.prototype;
assert(!("getCalendar" in PlainYearMonthProto), "Temporal.PlainYearMonth.prototype.getCalendar should not exist");
assert(!("getISOFields" in PlainYearMonthProto), "Temporal.PlainYearMonth.prototype.getISOFields should not exist");

const ZonedDateTimeProto = Temporal.ZonedDateTime.prototype;
assert(!("epochMicroseconds" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.epochMicroseconds should not exist");
assert(!("epochSeconds" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.epochSeconds should not exist");
assert(!("getCalendar" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.getCalendar should not exist");
assert(!("getISOFields" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.getISOFields should not exist");
assert(!("getTimeZone" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.getTimeZone should not exist");
assert(!("toPlainMonthDay" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.toPlainMonthDay should not exist");
assert(!("toPlainYearMonth" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.toPlainYearMonth should not exist");
assert(!("withPlainDate" in ZonedDateTimeProto), "Temporal.ZonedDateTime.prototype.withPlainDate should not exist");
