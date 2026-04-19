// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.equals
description: Calendar ID is canonicalized
features: [Temporal]
---*/

const instance = new Temporal.PlainDateTime(2024, 7, 2, 12, 34, 0, 0, 0, 0, "islamic-civil");

[
  "2024-07-02T12:34[u-ca=islamicc]",
  { year: 1445, month: 12, day: 25, hour: 12, minute: 34, calendar: "islamicc" },
].forEach((arg) => assert(instance.equals(arg), "calendar ID is canonicalized"));
