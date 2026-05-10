// Copyright (C) 2018 Bloomberg LP. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.plaindate.prototype.tolocalestring
description: Instant.toLocaleString with option timeZoneName outputs a short name.
features: [Temporal]
locale: [en-US]
---*/

const instant = Temporal.Instant.from("1976-11-18T14:23:30Z");
const str = instant.toLocaleString("en-US", {
  timeZone: "America/New_York"
});
const strWithName = instant.toLocaleString("en-US", {
  timeZone: "America/New_York",
  timeZoneName: "short"
});

assert(str.length < strWithName.length,
       'expected "' + str + '" to be shorter than "' + strWithName + '".');
