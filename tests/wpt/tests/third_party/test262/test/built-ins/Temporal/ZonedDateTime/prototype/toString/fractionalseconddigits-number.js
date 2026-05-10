// Copyright (C) 2022 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Number for fractionalSecondDigits option
features: [BigInt, Temporal]
---*/

const fewSeconds = new Temporal.ZonedDateTime(192_258_181_000_000_000n, "UTC");
const zeroSeconds = new Temporal.ZonedDateTime(0n, "UTC");
const wholeSeconds = new Temporal.ZonedDateTime(30_000_000_000n, "UTC");
const subSeconds = new Temporal.ZonedDateTime(30_123_400_000n, "UTC");

assert.sameValue(fewSeconds.toString({ fractionalSecondDigits: 0 }), "1976-02-04T05:03:01+00:00[UTC]",
  "pads parts with 0");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 0 }), "1970-01-01T00:00:30+00:00[UTC]",
  "truncates 4 decimal places to 0");
assert.sameValue(zeroSeconds.toString({ fractionalSecondDigits: 2 }), "1970-01-01T00:00:00.00+00:00[UTC]",
  "pads zero seconds to 2 decimal places");
assert.sameValue(wholeSeconds.toString({ fractionalSecondDigits: 2 }), "1970-01-01T00:00:30.00+00:00[UTC]",
  "pads whole seconds to 2 decimal places");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 2 }), "1970-01-01T00:00:30.12+00:00[UTC]",
  "truncates 4 decimal places to 2");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 3 }), "1970-01-01T00:00:30.123+00:00[UTC]",
  "truncates 4 decimal places to 3");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 6 }), "1970-01-01T00:00:30.123400+00:00[UTC]",
  "pads 4 decimal places to 6");
assert.sameValue(zeroSeconds.toString({ fractionalSecondDigits: 7 }), "1970-01-01T00:00:00.0000000+00:00[UTC]",
  "pads zero seconds to 7 decimal places");
assert.sameValue(wholeSeconds.toString({ fractionalSecondDigits: 7 }), "1970-01-01T00:00:30.0000000+00:00[UTC]",
  "pads whole seconds to 7 decimal places");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 7 }), "1970-01-01T00:00:30.1234000+00:00[UTC]",
  "pads 4 decimal places to 7");
assert.sameValue(subSeconds.toString({ fractionalSecondDigits: 9 }), "1970-01-01T00:00:30.123400000+00:00[UTC]",
  "pads 4 decimal places to 9");
