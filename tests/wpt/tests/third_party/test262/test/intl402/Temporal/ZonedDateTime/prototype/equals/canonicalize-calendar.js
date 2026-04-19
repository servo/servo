// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.equals
description: Calendar ID is canonicalized
features: [Temporal]
---*/

const instance = new Temporal.ZonedDateTime(1719923640_000_000_000n, "UTC", "islamic-civil");

[
  "2024-07-02T12:34+00:00[UTC][u-ca=islamicc]",
  { year: 1445, month: 12, day: 25, hour: 12, minute: 34, calendar: "islamicc", timeZone: "UTC" },
].forEach((arg) => assert(instance.equals(arg), "calendar ID is canonicalized"));
