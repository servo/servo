// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime
description: Some Etc/GMT{+/-}{0}N timezones are valid, but not all
features: [Temporal, canonical-tz]
---*/

// "Etc/GMT-0" through "Etc/GMT-14" are OK

[0,1,2,3,4,5,6,7,8,9,10,11,12,13,14].forEach((n) => {
  let tz = "Etc/GMT-" + n;
  let instance = new Temporal.ZonedDateTime(0n, tz);
  assert.sameValue(
    instance.timeZoneId,
    tz,
    tz + " is a valid timezone"
  );
});

let gmtMinus24TZ = "Etc/GMT-24";
assert.throws(
  RangeError,
  () => { new Temporal.ZonedDateTime(0n, gmtMinus24TZ); },
  gmtMinus24TZ + " is an invalid timezone"
);

// "Etc/GMT-0N" is not OK (1 ≤ N ≤ 9)
[1,2,3,4,5,6,7,8,9].forEach((n) => {
  let tz = "Etc/GMT-0" + n;
  assert.throws(
    RangeError,
    () => { new Temporal.ZonedDateTime(0n, tz); },
    tz + " is an invalid timezone"
  );
});

// "Etc/GMT+0N" is not OK (0 ≤ N ≤ 9)
[0,1,2,3,4,5,6,7,8,9].forEach((n) => {
  let tz = "Etc/GMT+0" + n;
  assert.throws(
    RangeError,
    () => { new Temporal.ZonedDateTime(0n, tz); },
    tz + " is an invalid timezone"
    );
});

// Etc/GMT+0" through "Etc/GMT+12" are OK

[0,1,2,3,4,5,6,7,8,9,10,11,12].forEach((n) => {
  let tz = "Etc/GMT+" + n;
  let instance = new Temporal.ZonedDateTime(0n, tz);
  assert.sameValue(
    instance.timeZoneId,
    tz,
    tz + " is a valid timezone"
  );
});

let gmtPlus24TZ = "Etc/GMT+24";
assert.throws(
  RangeError,
  () => { new Temporal.ZonedDateTime(0n, gmtPlus24TZ); },
  gmtPlus24TZ + " is an invalid timezone"
);
