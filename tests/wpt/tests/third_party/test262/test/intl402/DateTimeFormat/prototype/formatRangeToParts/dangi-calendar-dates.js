// Copyright 2025 Google Inc, Igalia S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Check that Intl.DateTimeFormat.formatRangeToParts output matches snapshot data
locale: [en-US-u-ca-dangi]
---*/

const calendar = "dangi";

// verify that Intl.DateTimeFormat.formatRangeToParts output matches snapshot data
function compareFormatRangeToPartsSnapshot(isoString1, isoString2, expectedComponents1, expectedComponents2) {
  const date1 = new Date(isoString1);
  const date2 = new Date(isoString2);
  const formatter = new Intl.DateTimeFormat(`en-US-u-ca-${calendar}`, { timeZone: "UTC" });
  const actualComponents = formatter.formatRangeToParts(date1, date2);

  function findAll(list, sourceVal, isoString) {
    for (let [expectedType, expectedValue] of Object.entries(list)) {
      const part = actualComponents.find(({type, source}) => type === expectedType && source == sourceVal);
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

  findAll(expectedComponents1, "startRange", isoString1);
  findAll(expectedComponents2, "endRange", isoString2);
}

compareFormatRangeToPartsSnapshot("2000-01-01T00:00Z", "1900-01-01T00:00Z", {
  relatedYear: 1999,
  month: 11,
  day: 25
}, {
  relatedYear: 1899,
  month: 12,
  day: 1
});

compareFormatRangeToPartsSnapshot("1900-01-01T00:00Z", "2050-01-01T00:00Z", {
  relatedYear: 1899,
  month: 12,
  day: 1
}, {
  relatedYear: 2049,
  month: 12,
  day: 8
});
