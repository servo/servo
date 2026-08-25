// Copyright (C) 2024 Sosuke Suzuki. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.DurationFormat.prototype.format
description: >
  The separator isn't printed when style is digital, hoursDisplay is auto and hours value is zero
locale: [en-US]
features: [Intl.DurationFormat]
---*/

const df = new Intl.DurationFormat("en", {
  style: "digital",
  hoursDisplay: "auto",
});

const durations = [
  // basic zero hours
  [{ hours: 0, minutes: 0, seconds: 2 }, "00:02"],
  [{ hours: 0, minutes: 1, seconds: 2 }, "01:02"],
  [{ days: 1, hours: 0, minutes: 1, seconds: 2 }, "1 day, 01:02"],

  // without hours
  [{ minutes: 0, seconds: 2 }, "00:02"],
  [{ minutes: 1, seconds: 2 }, "01:02"],
  [{ days: 1,  minutes: 1, seconds: 2 }, "1 day, 01:02"],

  // negative sign
  [{ hours: 0, minutes: -1, seconds: -2 }, "-01:02"],
  [{ hours: 0, minutes: -1, seconds: -2 }, "-01:02"],
  [{ days: -1, hours: 0, minutes: -1, seconds: -2 }, "-1 day, 01:02"],
];

for (const [duration, expected] of durations) {
  assert.sameValue(df.format(duration), expected, `Duration is ${JSON.stringify(duration)}`);
}
