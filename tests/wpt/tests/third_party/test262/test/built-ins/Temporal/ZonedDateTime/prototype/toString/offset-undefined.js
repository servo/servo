// Copyright (C) 2021 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.protoype.tostring
description: Fallback value for offset option
info: |
    sec-getoption step 3:
      3. If _value_ is *undefined*, return _fallback_.
    sec-temporal-toshowoffsetoption step 1:
      1. Return ? GetOption(_normalizedOptions_, *"offset"*, « String », « *"auto"*, *"never"* », *"auto"*).
    sec-temporal.zoneddatetime.protoype.tostring step 8:
      8. Let _showOffset_ be ? ToShowOffsetOption(_options_).
features: [Temporal]
---*/

const datetime = new Temporal.ZonedDateTime(1_000_000_000_987_654_321n, "UTC");

const explicit = datetime.toString({ offset: undefined });
assert.sameValue(explicit, "2001-09-09T01:46:40.987654321+00:00[UTC]", "default offset option is auto");

// See options-object.js for {} and () => {}
