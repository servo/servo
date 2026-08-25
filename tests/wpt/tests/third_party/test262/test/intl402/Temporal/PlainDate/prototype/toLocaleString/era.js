// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    PlainDate should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

const date = new Temporal.PlainDate(2000, 5, 2, "gregory");

assert(date.toLocaleString("en", { era: "narrow" }).startsWith("5"), "toLocaleString on a PlainDate with era option should work");
