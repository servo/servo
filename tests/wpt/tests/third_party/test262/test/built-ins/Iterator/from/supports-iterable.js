// Copyright (C) 2023 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Iterator.from supports iterables
info: |
  Iterator.from ( O )

includes: [compareArray.js]
features: [iterator-helpers]
flags: []
---*/
assert.compareArray(Array.from(Iterator.from([0, 1, 2, 3])), [0, 1, 2, 3]);
assert.compareArray(Array.from(Iterator.from(new String('str'))), ['s', 't', 'r']);
