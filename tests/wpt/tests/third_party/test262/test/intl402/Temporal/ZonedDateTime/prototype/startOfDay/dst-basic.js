// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.startofday
description: Basic tests around DST
features: [Temporal]
includes: [temporalHelpers.js]
---*/

const dayBeforeDstStart = new Temporal.PlainDateTime(2000, 4, 1, 2, 30).toZonedDateTime("America/Vancouver");

TemporalHelpers.assertPlainDatesEqual(
  dayBeforeDstStart.startOfDay().toPlainDate(),
  dayBeforeDstStart.toPlainDate(),
  "Date before dst start");
TemporalHelpers.assertPlainTime(
  dayBeforeDstStart.startOfDay().toPlainTime(),
  0, 0, 0, 0, 0, 0, 0,
  "Time before dst start");

const dayAfterSamoaDateLineChange = Temporal.PlainDateTime.from("2011-12-31T22:00").toZonedDateTime("Pacific/Apia");
TemporalHelpers.assertPlainTime(
  dayAfterSamoaDateLineChange.startOfDay().toPlainTime(),
  0, 0, 0, 0, 0, 0, 0,
  "Time after Samoa's date line change.");
