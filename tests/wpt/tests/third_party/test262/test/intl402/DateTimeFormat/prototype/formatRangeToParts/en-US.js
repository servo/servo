// Copyright (C) 2019 the V8 project authors, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimerangepattern
description: Basic tests for the en-US output of formatRangeToParts()
info: |
  Intl.DateTimeFormat.prototype.formatRangeToParts ( startDate , endDate )

  8. Return ? FormatDateTimeRange(dtf, x, y).
locale: [en-US]
features: [Intl.DateTimeFormat-formatRange]
---*/

// Tolerate implementation variance by expecting consistency without being prescriptive.
// TODO: can we change tests to be less reliant on CLDR formats while still testing that
// Temporal and Intl are behaving as expected?
const usDateRangeSeparator = new Intl.DateTimeFormat("en-US", { dateStyle: "short" })
  .formatRangeToParts(1 * 86400 * 1000, 366 * 86400 * 1000)
  .find((part) => part.type === "literal" && part.source === "shared").value;

function* zip(a, b) {
  assert.sameValue(a.length, b.length);
  for (let i = 0; i < a.length; ++i) {
    yield [i, a[i], b[i]];
  }
}

function compare(actual, expected) {
  for (const [i, actualEntry, expectedEntry] of zip(actual, expected)) {
    assert.sameValue(actualEntry.type, expectedEntry.type, `type for entry ${i}`);
    assert.sameValue(actualEntry.value, expectedEntry.value, `value for entry ${i}`);
    assert.sameValue(actualEntry.source, expectedEntry.source, `source for entry ${i}`);
  }
}

const date1 = new Date("2019-01-03T00:00:00");
const date2 = new Date("2019-01-05T00:00:00");
const date3 = new Date("2019-03-04T00:00:00");
const date4 = new Date("2020-03-04T00:00:00");

let dtf = new Intl.DateTimeFormat("en-US");
compare(dtf.formatRangeToParts(date1, date1), [
  { type: "month", value: "1", source: "shared" },
  { type: "literal", value: "/", source: "shared" },
  { type: "day", value: "3", source: "shared" },
  { type: "literal", value: "/", source: "shared" },
  { type: "year", value: "2019", source: "shared" },
]);
compare(dtf.formatRangeToParts(date1, date2), [
  { type: "month", value: "1", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "3", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "1", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "5", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2019", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date1, date3), [
  { type: "month", value: "1", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "3", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "3", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2019", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date1, date4), [
  { type: "month", value: "1", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "3", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "3", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2020", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date2, date3), [
  { type: "month", value: "1", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "5", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "3", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2019", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date2, date4), [
  { type: "month", value: "1", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "5", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "3", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2020", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date3, date4), [
  { type: "month", value: "3", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "day", value: "4", source: "startRange" },
  { type: "literal", value: "/", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "3", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: "/", source: "endRange" },
  { type: "year", value: "2020", source: "endRange" },
]);

dtf = new Intl.DateTimeFormat("en-US", {year: "numeric", month: "short", day: "numeric"});
compare(dtf.formatRangeToParts(date1, date1), [
  { type: "month", value: "Jan", source: "shared" },
  { type: "literal", value: " ", source: "shared" },
  { type: "day", value: "3", source: "shared" },
  { type: "literal", value: ", ", source: "shared" },
  { type: "year", value: "2019", source: "shared" },
]);
compare(dtf.formatRangeToParts(date1, date2), [
  { type: "month", value: "Jan", source: "shared" },
  { type: "literal", value: " ", source: "shared" },
  { type: "day", value: "3", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "day", value: "5", source: "endRange" },
  { type: "literal", value: ", ", source: "shared" },
  { type: "year", value: "2019", source: "shared" },
]);
compare(dtf.formatRangeToParts(date1, date3), [
  { type: "month", value: "Jan", source: "startRange" },
  { type: "literal", value: " ", source: "startRange" },
  { type: "day", value: "3", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "Mar", source: "endRange" },
  { type: "literal", value: " ", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: ", ", source: "shared" },
  { type: "year", value: "2019", source: "shared" },
]);
compare(dtf.formatRangeToParts(date1, date4), [
  { type: "month", value: "Jan", source: "startRange" },
  { type: "literal", value: " ", source: "startRange" },
  { type: "day", value: "3", source: "startRange" },
  { type: "literal", value: ", ", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "Mar", source: "endRange" },
  { type: "literal", value: " ", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: ", ", source: "endRange" },
  { type: "year", value: "2020", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date2, date3), [
  { type: "month", value: "Jan", source: "startRange" },
  { type: "literal", value: " ", source: "startRange" },
  { type: "day", value: "5", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "Mar", source: "endRange" },
  { type: "literal", value: " ", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: ", ", source: "shared" },
  { type: "year", value: "2019", source: "shared" },
]);
compare(dtf.formatRangeToParts(date2, date4), [
  { type: "month", value: "Jan", source: "startRange" },
  { type: "literal", value: " ", source: "startRange" },
  { type: "day", value: "5", source: "startRange" },
  { type: "literal", value: ", ", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "Mar", source: "endRange" },
  { type: "literal", value: " ", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: ", ", source: "endRange" },
  { type: "year", value: "2020", source: "endRange" },
]);
compare(dtf.formatRangeToParts(date3, date4), [
  { type: "month", value: "Mar", source: "startRange" },
  { type: "literal", value: " ", source: "startRange" },
  { type: "day", value: "4", source: "startRange" },
  { type: "literal", value: ", ", source: "startRange" },
  { type: "year", value: "2019", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "month", value: "Mar", source: "endRange" },
  { type: "literal", value: " ", source: "endRange" },
  { type: "day", value: "4", source: "endRange" },
  { type: "literal", value: ", ", source: "endRange" },
  { type: "year", value: "2020", source: "endRange" },
]);
