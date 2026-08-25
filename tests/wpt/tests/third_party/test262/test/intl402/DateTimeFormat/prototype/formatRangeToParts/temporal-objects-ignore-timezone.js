// Copyright (C) 2026 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-datetime-format-functions
description: Intl.DateTimeFormat.prototype.formatRangeToParts ignores timezone when isPlain is true.
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

function find_part(parts, expected_type, expected_source) {
  return parts.find(({ type, source }) =>
    type === expected_type && (source === expected_source || source === "shared")
  ).value;
}

// PlainDateTime
assert.sameValue(
  find_part(dtf_apia.formatRangeToParts(pdt_apia, pdt_los_angeles), "day", "startRange"),
  "30",
  "day is calculated correctly, ignoring the Pacific/Apia timezone"
);

assert.sameValue(
  find_part(dtf_los_angeles.formatRangeToParts(pdt_apia, pdt_los_angeles), "hour", "endRange"),
  "2",
  "hour is calculated correctly with the America/Los_Angeles timezone"
);

// PlainDate
assert.sameValue(
  find_part(dtf_apia.formatRangeToParts(pd_apia, pd_los_angeles), "day", "startRange"),
  "30",
  "day is calculated correctly, ignoring the Pacific/Apia timezone"
);

// PlainTime
assert.sameValue(
  find_part(dtf_los_angeles.formatRangeToParts(pt_apia, pt_los_angeles), "hour", "endRange"),
  "2",
  "hour is calculated correctly with the America/Los_Angeles timezone"
);
