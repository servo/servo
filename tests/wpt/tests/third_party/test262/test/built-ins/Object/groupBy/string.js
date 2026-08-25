// Copyright (c) 2023 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-object.groupby
description: Object.groupBy works for string items
info: |
  Object.groupBy ( items, callbackfn )
  ...
includes: [compareArray.js]
features: [array-grouping]
---*/

const string = '🥰💩🙏😈';

const obj = Object.groupBy(string, function (char) {
  return char < '🙏' ? 'before' : 'after';
});

assert.compareArray(Object.keys(obj), ['after', 'before']);
assert.compareArray(obj.before, ['💩', '😈']);
assert.compareArray(obj.after, ['🥰', '🙏']);
