// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.withcalendar
description: TypeError thrown when calendar argument not given
features: [Temporal]
---*/

const plainDate = Temporal.PlainDate.from("1976-11-18");
assert.throws(TypeError, () => plainDate.withCalendar(), "missing argument");
assert.throws(TypeError, () => plainDate.withCalendar(undefined), "undefined argument");
