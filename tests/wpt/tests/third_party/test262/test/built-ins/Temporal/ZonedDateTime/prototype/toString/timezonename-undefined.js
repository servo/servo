// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.protoype.tostring
description: Fallback value for timeZoneName option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-toshowtimezonenameoption step 1:
      1. Return ? GetOption(_normalizedOptions_, *"timeZoneName"*, « *"string"* », « *"auto"*, *"never"*, *"critical"* », *"auto"*).
    sec-temporal.zoneddatetime.protoype.tostring step 7:
      7. Let _showTimeZone_ be ? ToShowTimeZoneNameOption(_options_).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");

const explicit = datetime.toString({ timeZoneName: undefined });
assert.sameValue(explicit, "2001-09-09T01:46:40.987654321+00:00[UTC]", "default timeZoneName option is auto");

// See options-object.js for {} and () => {}
