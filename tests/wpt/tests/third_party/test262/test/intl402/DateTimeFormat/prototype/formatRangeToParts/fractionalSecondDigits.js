// Copyright 2020 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-createdatetimeformat
description: Checks basic handling of fractionalSecondDigits.
features: [Intl.DateTimeFormat-fractionalSecondDigits, Intl.DateTimeFormat-formatRange]
locale: [en-US]
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

const d1 = new Date(2019, 7, 10,  1, 2, 3, 234);
const d2 = new Date(2019, 7, 10,  1, 2, 3, 567);
const d3 = new Date(2019, 7, 10,  1, 2, 13, 987);

assert.throws(RangeError, () => {
    new Intl.DateTimeFormat(
      'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 0});
  }, "fractionalSecondDigits 0 should throw RangeError for out of range");

assert.throws(RangeError, () => {
    new Intl.DateTimeFormat(
      'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 4});
  }, "fractionalSecondDigits 4 should throw RangeError for out of range");

let dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: undefined});

compare(dtf.formatRangeToParts(d1, d2), [
  { type: "minute", value: "02", source: "shared" },
  { type: "literal", value: ":", source: "shared" },
  { type: "second", value: "03", source: "shared" }
]);

compare(dtf.formatRangeToParts(d1, d3), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "13", source: "endRange" }
]);

dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 1});

compare(dtf.formatRangeToParts(d1, d2), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: ".", source: "startRange" },
  { type: "fractionalSecond", value: "2", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "03", source: "endRange" },
  { type: "literal", value: ".", source: "endRange" },
  { type: "fractionalSecond", value: "5", source: "endRange" }
]);

compare(dtf.formatRangeToParts(d1, d3), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: ".", source: "startRange" },
  { type: "fractionalSecond", value: "2", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "13", source: "endRange" },
  { type: "literal", value: ".", source: "endRange" },
  { type: "fractionalSecond", value: "9", source: "endRange" }
]);

dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 2});

compare(dtf.formatRangeToParts(d1, d2), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: ".", source: "startRange" },
  { type: "fractionalSecond", value: "23", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "03", source: "endRange" },
  { type: "literal", value: ".", source: "endRange" },
  { type: "fractionalSecond", value: "56", source: "endRange" }
]);

compare(dtf.formatRangeToParts(d1, d3), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: ".", source: "startRange" },
  { type: "fractionalSecond", value: "23", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "13", source: "endRange" },
  { type: "literal", value: ".", source: "endRange" },
  { type: "fractionalSecond", value: "98", source: "endRange" }
]);

dtf = new Intl.DateTimeFormat(
    'en', { minute: "numeric", second: "numeric", fractionalSecondDigits: 3});

compare(dtf.formatRangeToParts(d1, d2), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: ".", source: "startRange" },
  { type: "fractionalSecond", value: "234", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "03", source: "endRange" },
  { type: "literal", value: ".", source: "endRange" },
  { type: "fractionalSecond", value: "567", source: "endRange" }
]);

compare(dtf.formatRangeToParts(d1, d3), [
  { type: "minute", value: "02", source: "startRange" },
  { type: "literal", value: ":", source: "startRange" },
  { type: "second", value: "03", source: "startRange" },
  { type: "literal", value: ".", source: "startRange" },
  { type: "fractionalSecond", value: "234", source: "startRange" },
  { type: "literal", value: usDateRangeSeparator, source: "shared" },
  { type: "minute", value: "02", source: "endRange" },
  { type: "literal", value: ":", source: "endRange" },
  { type: "second", value: "13", source: "endRange" },
  { type: "literal", value: ".", source: "endRange" },
  { type: "fractionalSecond", value: "987", source: "endRange" }
]);

