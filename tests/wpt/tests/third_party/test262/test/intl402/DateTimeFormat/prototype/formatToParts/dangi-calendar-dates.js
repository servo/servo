// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Check that Intl.DateTimeFormat.formatToParts output matches snapshot data
locale: [en-US-u-ca-dangi]
features: [Temporal]
---*/

const calendar = "dangi";

const dangiYearOffset = new Temporal.PlainDate(1, 1, 1, calendar).year;

// verify that Intl.DateTimeFormat.formatToParts output matches snapshot data
function compareFormatToPartsSnapshot(isoString, expectedComponents) {
  const date = new Date(isoString);
  const formatter = new Intl.DateTimeFormat(`en-US-u-ca-${calendar}`, { timeZone: "UTC" });
  const actualComponents = formatter.formatToParts(date);
  for (let [expectedType, expectedValue] of Object.entries(expectedComponents)) {
    const part = actualComponents.find(({type}) => type === expectedType);
    const contextMessage = `${expectedType} component of ${isoString} formatted in ${calendar}`;
    assert.notSameValue(part, undefined, `${contextMessage} is missing`);
    assert.sameValue(typeof part.value, "string", `${contextMessage} is not a string`);
    assert(
      part.value === String(expectedValue) ||
        part.value === String(expectedValue).padStart(2, "0"),
      `${contextMessage} has unexpected value`
    );
  }
}

compareFormatToPartsSnapshot("2000-01-01T00:00Z", {
  relatedYear: 1999,
  month: 11,
  day: 25,
});

compareFormatToPartsSnapshot("1900-01-01T00:00Z", {
  relatedYear: 1899,
  month: 12,
  day: 1,
});

compareFormatToPartsSnapshot("2050-01-01T00:00Z", {
  relatedYear: 2049,
  month: 12,
  day: 8,
});
