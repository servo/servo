// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plainyearmonth.prototype.tolocalestring
description: Calendar must match the locale calendar
features: [Temporal, Intl-enumeration]
---*/

const localeCalendar = new Intl.DateTimeFormat().resolvedOptions().calendar;
assert.notSameValue(localeCalendar, "iso8601", "no locale has the ISO calendar");

const sameCalendarInstance = new Temporal.PlainDate(2000, 1, 1, localeCalendar).toPlainYearMonth();
const result = sameCalendarInstance.toLocaleString();
assert.sameValue(typeof result, "string", "toLocaleString() succeeds when instance has the same calendar as locale");

// Pick a different calendar that is not ISO and not the locale's calendar
const calendars = new Set(Intl.supportedValuesOf("calendar"));
calendars.delete("iso8601");
calendars.delete(localeCalendar);
const differentCalendar = calendars.values().next().value;

const differentCalendarInstance = new Temporal.PlainDate(2000, 1, 1, differentCalendar).toPlainYearMonth();
assert.throws(RangeError, () => differentCalendarInstance.toLocaleString(), "calendar mismatch");

const isoInstance = new Temporal.PlainDate(2000, 1, 1, "iso8601").toPlainYearMonth();
assert.throws(RangeError, () => isoInstance.toLocaleString(), "calendar mismatch even when instance has the ISO calendar")
