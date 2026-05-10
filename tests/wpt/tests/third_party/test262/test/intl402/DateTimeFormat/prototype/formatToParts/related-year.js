// Copyright 2019 Google Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-partitiondatetimepattern
description: >
  Checks the output of 'relatedYear' and 'yearName' type, and
  the choose of pattern base on calendar.
locale: [en-u-ca-chinese]
---*/

let df = new Intl.DateTimeFormat("en-u-ca-chinese", {year: "numeric"});
let parts = df.formatToParts(new Date());
var relatedYearCount = 0;
var yearNameCount = 0;
parts.forEach(function(part) {
  relatedYearCount += (part.type == "relatedYear") ? 1 : 0;
  yearNameCount += (part.type == "yearName") ? 1 : 0;
});
assert.sameValue(relatedYearCount > 0, true);
assert.sameValue(yearNameCount > 0, true);
