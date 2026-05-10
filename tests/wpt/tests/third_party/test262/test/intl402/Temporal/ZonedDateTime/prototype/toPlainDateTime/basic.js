// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.toplaindatetime
description: Sample of results for IANA time zones
includes: [temporalHelpers.js]
features: [Temporal]
---*/

function test(epochNs, results) {
  Object.entries(results).forEach(([id, expected]) => {
    const instance = new Temporal.ZonedDateTime(epochNs, id);
    const dt = instance.toPlainDateTime();
    TemporalHelpers.assertPlainDateTime(dt, ...expected, `Local time of ${instance.toInstant()} in ${id}`);
  });
}

// Unix epoch
test(0n, {
  'America/Los_Angeles': [1969, 12, "M12", 31, 16, 0, 0, 0, 0, 0],
  'America/New_York': [1969, 12, "M12", 31, 19, 0, 0, 0, 0, 0],
  'Africa/Monrovia': [1969, 12, "M12", 31, 23, 15, 30, 0, 0, 0],
  'Europe/London': [1970, 1, "M01", 1, 1, 0, 0, 0, 0, 0],
  'Europe/Berlin': [1970, 1, "M01", 1, 1, 0, 0, 0, 0, 0],
  'Europe/Moscow': [1970, 1, "M01", 1, 3, 0, 0, 0, 0, 0],
  'Asia/Kolkata': [1970, 1, "M01", 1, 5, 30, 0, 0, 0, 0],
  'Asia/Tokyo': [1970, 1, "M01", 1, 9, 0, 0, 0, 0, 0],
});

// Just before epoch
test(-1n, {
  'America/Los_Angeles': [1969, 12, "M12", 31, 15, 59, 59, 999, 999, 999],
  'America/New_York': [1969, 12, "M12", 31, 18, 59, 59, 999, 999, 999],
  'Africa/Monrovia': [1969, 12, "M12", 31, 23, 15, 29, 999, 999, 999],
  'Europe/London': [1970, 1, "M01", 1, 0, 59, 59, 999, 999, 999],
  'Europe/Berlin': [1970, 1, "M01", 1, 0, 59, 59, 999, 999, 999],
  'Europe/Moscow': [1970, 1, "M01", 1, 2, 59, 59, 999, 999, 999],
  'Asia/Kolkata': [1970, 1, "M01", 1, 5, 29, 59, 999, 999, 999],
  'Asia/Tokyo': [1970, 1, "M01", 1, 8, 59, 59, 999, 999, 999],
});

// Just after epoch
test(1n, {
  'America/Los_Angeles': [1969, 12, "M12", 31, 16, 0, 0, 0, 0, 1],
  'America/New_York': [1969, 12, "M12", 31, 19, 0, 0, 0, 0, 1],
  'Africa/Monrovia': [1969, 12, "M12", 31, 23, 15, 30, 0, 0, 1],
  'Europe/London': [1970, 1, "M01", 1, 1, 0, 0, 0, 0, 1],
  'Europe/Berlin': [1970, 1, "M01", 1, 1, 0, 0, 0, 0, 1],
  'Europe/Moscow': [1970, 1, "M01", 1, 3, 0, 0, 0, 0, 1],
  'Asia/Kolkata': [1970, 1, "M01", 1, 5, 30, 0, 0, 0, 1],
  'Asia/Tokyo': [1970, 1, "M01", 1, 9, 0, 0, 0, 0, 1],
});

// Hours before epoch
test(-6300_000_000_001n, {
  'America/Los_Angeles': [1969, 12, "M12", 31, 14, 14, 59, 999, 999, 999],
  'America/New_York': [1969, 12, "M12", 31, 17, 14, 59, 999, 999, 999],
  'Africa/Monrovia': [1969, 12, "M12", 31, 21, 30, 29, 999, 999, 999],
  'Europe/London': [1969, 12, "M12", 31, 23, 14, 59, 999, 999, 999],
  'Europe/Berlin': [1969, 12, "M12", 31, 23, 14, 59, 999, 999, 999],
  'Europe/Moscow': [1970, 1, "M01", 1, 1, 14, 59, 999, 999, 999],
  'Asia/Kolkata': [1970, 1, "M01", 1, 3, 44, 59, 999, 999, 999],
  'Asia/Tokyo': [1970, 1, "M01", 1, 7, 14, 59, 999, 999, 999],
});

// Hours after epoch
test(6300_000_000_001n, {
  'America/Los_Angeles': [1969, 12, "M12", 31, 17, 45, 0, 0, 0, 1],
  'America/New_York': [1969, 12, "M12", 31, 20, 45, 0, 0, 0, 1],
  'Africa/Monrovia': [1970, 1, "M01", 1, 1, 0, 30, 0, 0, 1],
  'Europe/London': [1970, 1, "M01", 1, 2, 45, 0, 0, 0, 1],
  'Europe/Berlin': [1970, 1, "M01", 1, 2, 45, 0, 0, 0, 1],
  'Europe/Moscow': [1970, 1, "M01", 1, 4, 45, 0, 0, 0, 1],
  'Asia/Kolkata': [1970, 1, "M01", 1, 7, 15, 0, 0, 0, 1],
  'Asia/Tokyo': [1970, 1, "M01", 1, 10, 45, 0, 0, 0, 1],
});
