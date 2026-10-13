// Copyright (C) 2022 Rudolph Gottesheim. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-formatdatetime
description: >
    Short German weekdays should be followed by a comma.
locale: [de]
---*/

let dtf = new Intl.DateTimeFormat("de", { weekday: "short", day: "numeric", month: "long"});
assert.sameValue(dtf.format(new Date(2022, 11, 24)).includes("Sa.,"), true, "German weekday without year");
