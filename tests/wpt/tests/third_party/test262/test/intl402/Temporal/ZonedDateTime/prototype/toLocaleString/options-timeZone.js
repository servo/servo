// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tolocalestring
description: >
  Options must not have a timeZone property, even if it agrees with the
  instance's time zone
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(0n, "UTC");

assert.throws(TypeError, () => datetime.toLocaleString("en-US", { timeZone: "Europe/Vienna" }), "timeZone option disallowed");
assert.throws(TypeError, () => datetime.toLocaleString("en-US", { timeZone: "UTC" }), "timeZone option disallowed even if it agrees with instance's time zone");
