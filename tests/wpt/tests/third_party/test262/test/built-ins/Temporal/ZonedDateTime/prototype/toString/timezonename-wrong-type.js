// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.protoype.tostring
description: Type conversions for timeZoneName option
info: |
    sec-getoption step 9.a:
      a. Set _value_ to ? ToString(_value_).
    sec-temporal-toshowtimezonenameoption step 1:
      1. Return ? GetOption(_normalizedOptions_, *"timeZoneName"*, « *"string"* », « *"auto"*, *"never"*, *"critical"* », *"auto"*).
    sec-temporal.zoneddatetime.protoype.tostring step 7:
      7. Let _showTimeZone_ be ? ToShowTimeZoneNameOption(_options_).
includes: [compareArray.js, temporalHelpers.js]
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");

TemporalHelpers.checkStringOptionWrongType("timeZoneName", "auto",
  (timeZoneName) => datetime.toString({ timeZoneName }),
  (result, descr) => assert.sameValue(result, "2001-09-09T01:46:40.987654321+00:00[UTC]", descr),
);
