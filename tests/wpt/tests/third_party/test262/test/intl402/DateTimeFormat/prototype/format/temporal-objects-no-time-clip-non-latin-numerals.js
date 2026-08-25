// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Temporal dates outside the TimeClip range are correctly formatted when the
  locale uses non-Latin numerals (e.g., Arabic-Indic digits).
info: |
  When a Temporal plain object is outside the range representable by a legacy
  Date and the locale's numbering system uses non-Latin digits, the
  implementation must re-format the adjusted day value in the correct numeral
  system.
features: [Temporal]
locale: [ar-EG]
---*/

// Arabic-Egyptian locale uses Arabic-Indic digits (U+0660..U+0669)
var dtf = new Intl.DateTimeFormat("ar-EG", {
  year: "numeric",
  month: "numeric",
  day: "numeric",
  calendar: "iso8601"
});

// Maximum plain date value: +275760-09-13
// This is outside the TimeClip range, so the day needs adjustment.
var maxDate = new Temporal.PlainDate(275760, 9, 13);
var result = dtf.format(maxDate);

// The result should contain the day 13 in Arabic-Indic digits: ١٣
assert(result.includes("\u0661\u0663"), "max date should include day 13 in Arabic-Indic digits, got: " + result);
// The result should contain the month 9 in Arabic-Indic digits: ٩
assert(result.includes("\u0669"), "max date should include month 9 in Arabic-Indic digit, got: " + result);

// Minimum plain date value: -271821-04-19
var minDate = new Temporal.PlainDate(-271821, 4, 19);
var minResult = dtf.format(minDate);

// The result should contain the day 19 in Arabic-Indic digits: ١٩
assert(minResult.includes("\u0661\u0669"), "min date should include day 19 in Arabic-Indic digits, got: " + minResult);
