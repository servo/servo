// Copyright (C) 2023 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-temporal.zoneddatetime.prototype.subtract
description: Hours overflow.
includes: [temporalHelpers.js]
features: [Temporal]
---*/

// subtract result
// var later = Temporal.ZonedDateTime.from("2019-10-29T10:46:38.271986102-03:00[-03:00]");
var later = new Temporal.ZonedDateTime(1572356798271986102n, "-03:00");
var earlier = later.subtract({ hours: 12 });

TemporalHelpers.assertZonedDateTimesEqual(
    earlier,
    // "2019-10-28T22:46:38.271986102-03:00[-03:00]"
    new Temporal.ZonedDateTime(1572313598271986102n, "-03:00"));

// "2020-05-31T23:12:38.271986102-04:00[-04:00]"
earlier = new Temporal.ZonedDateTime(1590981158271986102n, "-04:00");
later = new Temporal.ZonedDateTime(1590988358271986102n, "-04:00");

TemporalHelpers.assertZonedDateTimesEqual(earlier.subtract({ hours: -2 }), later);

