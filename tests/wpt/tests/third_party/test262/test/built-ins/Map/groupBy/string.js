// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-map.groupby
description: Map.groupBy works for string items
info: |
  Map.groupBy ( items, callbackfn )
  ...
includes: [compareArray.js]
features: [array-grouping, Map]
---*/

const string = '🥰💩🙏😈';

const map = Map.groupBy(string, function (char) {
  return char < '🙏' ? 'before' : 'after';
});

assert.compareArray(Array.from(map.keys()), ['after', 'before']);
assert.compareArray(map.get('before'), ['💩', '😈']);
assert.compareArray(map.get('after'), ['🥰', '🙏']);
