// Copyright 2024 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-formatnumerichours
description: >
  Test to ensure that useGrouping is set to off for hours, minutes, and seconds under digital style.

info: |
  1.1.9 FormatNumericHours ( durationFormat, hoursValue, signDisplayed )
  ...
  9. Perform ! CreateDataPropertyOrThrow(nfOpts, "useGrouping", false).

  1.1.10 FormatNumericMinutes ( durationFormat, minutesValue, hoursDisplayed, signDisplayed )
  ...
  10. Perform ! CreateDataPropertyOrThrow(nfOpts, "useGrouping", false).

  1.1.11 FormatNumericSeconds ( durationFormat, secondsValue, minutesDisplayed, signDisplayed )
  ...
  9. Perform ! CreateDataPropertyOrThrow(nfOpts, "useGrouping", false).

locale: [en]
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat("en", {style: "digital"});

assert.sameValue(df.format({hours: 1234567, minutes: 20, seconds: 45}),
    "1234567:20:45", `format in digit set useGrouping to false`);

assert.sameValue(df.format({hours: 12, minutes: 1234567, seconds: 20}),
    "12:1234567:20", `format in digit set useGrouping to false`);

assert.sameValue(df.format({hours: 12, minutes: 34, seconds: 1234567}),
    "12:34:1234567", `format in digit set useGrouping to false`);

assert.sameValue(df.format({hours: 12, minutes: 34, seconds: 56, milliseconds: 1234567}),
    "12:34:1290.567", `format in digit set useGrouping to false`);

assert.sameValue(df.format({days: 1234567, hours: 3, minutes: 20, seconds: 45}),
    "1,234,567 days, 3:20:45", `useGrouping set to false applies to time units but not days`);

assert.sameValue(df.format({days: 1234567, hours: 2345678, minutes: 3456789, seconds: 4567890}),
    "1,234,567 days, 2345678:3456789:4567890", `useGrouping set to false applies to time units but not days`);
