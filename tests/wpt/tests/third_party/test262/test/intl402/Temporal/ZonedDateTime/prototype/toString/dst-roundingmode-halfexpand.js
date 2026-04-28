// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.tostring
description: Test behaviour around DST boundaries with the option roundingMode set to halfExpand
features: [Temporal]
---*/

var zdtNonExistent = Temporal.PlainDateTime.from("2000-04-02T01:59:59.999999999").toZonedDateTime("America/Vancouver");
var roundedString = zdtNonExistent.toString({
  fractionalSecondDigits: 8,
  roundingMode: "halfExpand"
});
assert.sameValue(roundedString, "2000-04-02T03:00:00.00000000-07:00[America/Vancouver]");
var instant = Temporal.Instant.from(roundedString);
assert.sameValue(instant.epochNanoseconds - zdtNonExistent.epochNanoseconds, 1n);
