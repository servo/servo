// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Intl.DateTimeFormat.prototype.formatRange ignores timezone when isPlain is true.
features: [Temporal]
---*/

// Non existent date in the Pacific/Apia timezone.
const datetime_apia = '2011-12-30T12:00:00';
// Non existent time in the America/Los_Angeles timezone.
const datetime_los_angeles = '2026-03-08T02:00:00';

const pdt_apia = Temporal.PlainDateTime.from(datetime_apia);
const pdt_los_angeles = Temporal.PlainDateTime.from(datetime_los_angeles);

const pd_apia = Temporal.PlainDate.from(datetime_apia);
const pd_los_angeles = Temporal.PlainDate.from(datetime_los_angeles);

const pt_apia = Temporal.PlainTime.from(datetime_apia);
const pt_los_angeles = Temporal.PlainTime.from(datetime_los_angeles);

const dtf_apia = new Intl.DateTimeFormat('en-US', { dateStyle: 'short', timeStyle: 'short', timeZone: 'Pacific/Apia' });
const dtf_los_angeles = new Intl.DateTimeFormat('en-US', { dateStyle: 'short', timeStyle: 'short', timeZone: 'America/Los_Angeles' });

// PlainDateTime
const pdt_apia_result = dtf_apia.formatRange(pdt_apia, pdt_los_angeles);
assert(
  pdt_apia_result.includes('30') && !pdt_apia_result.includes('31'),
  "day is calculated correctly, ignoring the Pacific/Apia timezone"
);

const pdt_los_angeles_result = dtf_los_angeles.formatRange(pdt_apia, pdt_los_angeles);
assert(
  pdt_los_angeles_result.includes('2:00') && !pdt_los_angeles_result.includes('3:00'),
  "hour is calculated correctly with the America/Los_Angeles timezone"
);

// PlainDate
const pd_apia_result = dtf_apia.formatRange(pd_apia, pd_los_angeles);
assert(
  pd_apia_result.includes('30') && !pd_apia_result.includes('31'),
  "day is calculated correctly, ignoring the Pacific/Apia timezone"
);

// PlainTime
const pt_los_angeles_result = dtf_los_angeles.formatRange(pt_apia, pt_los_angeles);
assert(
  pt_los_angeles_result.includes('2:00') && !pt_los_angeles_result.includes('3:00'),
  "hour is calculated correctly with the America/Los_Angeles timezone"
);
