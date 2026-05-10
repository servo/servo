// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tozoneddatetime
description: >
  Accessor property for "timeZone" throws an error.
info: |
  Temporal.PlainDate.prototype.toZonedDateTime ( item )

  ...
  3. If item is an Object, then
    a. Let timeZoneLike be ? Get(item, "timeZone").
  ...
features: [Temporal]
---*/

var instance = new Temporal.PlainDate(1970, 1, 1);

var item = {
  get timeZone() {
    throw new Test262Error();
  }
};

assert.throws(Test262Error, () => instance.toZonedDateTime(item));
