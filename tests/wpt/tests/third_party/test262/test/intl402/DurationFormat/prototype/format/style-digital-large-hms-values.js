// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-formatnumericunits
description: >
  Test to ensure that grouping separators are suppressed when formatting Durations with /hour, minute, and second values while using digital style.

info: |
  1.1.12 FormatNumericUnits ( durationFormat, duration, firstNumericUnit, signDisplayed )

  16. If hoursFormatted is true, then
    ...
    b. Append FormatNumericHours ( durationFormat, hoursValue, signDisplayed ) to numericPartsList.
  17. If minutesFormatted is true, then
    ...
    b. Append FormatNumericMinutes ( durationFormat, minutesValue, hoursFormatted, signDisplayed ) to numericPartsList.
  18. If secondsFormatted is true, then
    a. Append FormatNumericSeconds ( durationFormat, secondsValue, minutesFormatted, signDisplayed ) to numericPartsList.

  1.1.9 FormatNumericHours ( durationFormat, hoursValue, signDisplayed )
    ...
    9. Perform ! CreateDataPropertyOrThrow(nfOpts, "useGrouping", false).

  1.1.10 FormatNumericMinutes (durationFormat, minutesValue, hoursDisplayed, signDisplayed)
    ...
    10. Perform ! CreateDataPropertyOrThrow(nfOpts, "useGrouping", false).

  1.1.11 FormatNumericSeconds ( durationFormat, secondsValue, minutesDisplayed, signDisplayed )
    ...
    10. Perform ! CreateDataPropertyOrThrow(nfOpts, "useGrouping", false).

locale: [en]
features: [Intl.DurationFormat]
---*/

const locale = "en";
const duration = {hours: 1234, minutes: 1234567, seconds: 12345678};

let df = new Intl.DurationFormat(locale, {style: "digital"});

assert.sameValue(df.format(duration), "1234:1234567:12345678", `failed to suppress grouping separator using digital style`);
