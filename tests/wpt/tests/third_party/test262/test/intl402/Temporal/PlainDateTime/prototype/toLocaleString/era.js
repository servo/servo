// Copyright (C) 2025 Igalia, S.L.. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindatetime.prototype.tolocalestring
description: >
    If era option and no other options are provided to toLocaleString,
    PlainDateTime should be foramtted with default options
features: [Temporal]
locale: [en]
---*/

var date = new Temporal.PlainDateTime(2000, 5, 2, 14, 46, 0, 0, 0, 0, "gregory");

assert(date.toLocaleString("en", { era: "narrow" }).startsWith("5"), "toLocaleString on a PlainDateTime with era option should work");
