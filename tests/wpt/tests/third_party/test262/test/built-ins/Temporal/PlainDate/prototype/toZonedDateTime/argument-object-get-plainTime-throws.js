// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  Accessor property for "plainTime" throws an error.
info: |
  Temporal.PlainDate.prototype.toZonedDateTime ( item )

  ...
  3. If item is an Object, then
    a. Let timeZoneLike be ? Get(item, "timeZone").
    b. If timeZoneLike is undefined, then
      ...
    c. Else,
      i. Let timeZone be ? ToTemporalTimeZoneIdentifier(timeZoneLike).
      ii. Let temporalTime be ? Get(item, "plainTime").
  ...
features: [Temporal]
---*/

var instance = new Temporal.PlainDate(1970, 1, 1);

var item = {
  timeZone: "UTC",
  get plainTime() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => instance.toZonedDateTime(item));
