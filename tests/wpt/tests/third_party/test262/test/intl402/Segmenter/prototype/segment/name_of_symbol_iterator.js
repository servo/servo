// Copyright 2025 by Google, Inc.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-Intl.Segmenter.prototype.segment
description: Checks the "name" property of %IntlSegmentsPrototype%[%Symbol.iterator%] 
features: [Intl.Segmenter]
---*/
let ss = (new Intl.Segmenter()).segment("123")[Symbol.iterator].name;
assert.sameValue("[Symbol.iterator]", ss);
