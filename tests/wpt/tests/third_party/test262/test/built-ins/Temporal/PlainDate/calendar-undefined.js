// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate
description: Calendar argument defaults to the built-in ISO 8601 calendar
features: [Temporal]
---*/

const args = [2020, 12, 24];

const dateExplicit = new Temporal.PlainDate(...args, undefined);
assert.sameValue(dateExplicit.calendarId, "iso8601", "calendar string is iso8601");

const dateImplicit = new Temporal.PlainDate(...args);
assert.sameValue(dateImplicit.calendarId, "iso8601", "calendar string is iso8601");
