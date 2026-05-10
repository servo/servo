// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-assertion
es6id: 21.2.2.6
description: The `y` flag has no effect on the `^` assertion
info: |
  Even when the y flag is used with a pattern, ^ always matches only at the
  beginning of Input, or (if Multiline is true) at the beginning of a line.
---*/

var re;

re = /^a/y;

re.lastIndex = 0;
assert.sameValue(
  re.test('a'), true, 'positive: beginning of input (without `m`)'
);

re.lastIndex = 1;
assert.sameValue(
  re.test(' a'), false, 'negative: within a line (without `m`)'
);

re.lastIndex = 1;
assert.sameValue(
  re.test('\na'), false, 'negative: beginning of line (without `m`)'
);

re = /^a/my;

re.lastIndex = 0;
assert.sameValue(
  re.test('a'), true, 'positive: beginning of input (with `m`)'
);

re.lastIndex = 1;
assert.sameValue(
  re.test(' a'), false, 'negative: within a line (with `m`)'
);

re.lastIndex = 1;
assert.sameValue(
  re.test('\na'), true, 'positive: beginning of line (with `m`)'
);
