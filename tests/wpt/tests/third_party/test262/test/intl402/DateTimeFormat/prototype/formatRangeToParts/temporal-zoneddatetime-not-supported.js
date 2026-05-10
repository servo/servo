// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-intl.datetimeformat.prototype.formatRange
description: Temporal.ZonedDateTime is not supported directly in formatRange()
features: [Temporal]
---*/

const formatter = new Intl.DateTimeFormat();

// Check that TypeError would not be thrown for a different reason
const {timeZone, ...options} = formatter.resolvedOptions();
const datetime1 = new Temporal.ZonedDateTime(0n, timeZone);
assert.sameValue(typeof datetime1.toLocaleString(undefined, options), "string", "toLocaleString() with same options succeeds");

const datetime2 = new Temporal.ZonedDateTime(1_000_000_000n, timeZone);
assert.throws(TypeError, () => formatter.formatRange(datetime1, datetime2), "formatRange() does not support Temporal.ZonedDateTime");
