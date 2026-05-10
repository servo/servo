// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    ZonedDateTime should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

const zdt = new Temporal.ZonedDateTime(0n, "UTC");

assert(zdt.toLocaleString("en", { era: "narrow" }).startsWith("1"), "toLocaleString on a ZonedDateTime with era option should work");
