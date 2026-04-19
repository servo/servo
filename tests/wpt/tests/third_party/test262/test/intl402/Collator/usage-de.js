// Copyright 2018 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-initializecollator
description: Checks the behavior of search and sort in German.
includes: [compareArray.js]
locale: [de]
---*/

assert.compareArray(["AE", "\u00C4"].sort(new Intl.Collator("de", {usage: "sort"}).compare),
                    ["\u00C4", "AE"],
                    "sort");
assert.compareArray(["AE", "\u00C4"].sort(new Intl.Collator("de", {usage: "search"}).compare),
                    ["AE", "\u00C4"],
                    "search");
