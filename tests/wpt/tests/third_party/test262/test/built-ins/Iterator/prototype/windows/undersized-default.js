// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  When undersized is undefined or omitted, it defaults to "only-full" behavior
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  5. If undersized is undefined, set undersized to "only-full".

features: [iterator-chunking, generators]
---*/
function* g() {
  yield 0;
  yield 1;
  yield 2;
  yield 3;
  yield 4;
  yield 5;
}

// With windowSize larger than iterator, "only-full" yields nothing
let result;

result = Array.from(g().windows(100));
assert.sameValue(result.length, 0, 'omitted undersized defaults to "only-full"');

result = Array.from(g().windows(100, undefined));
assert.sameValue(result.length, 0, 'explicit undefined defaults to "only-full"');

result = Array.from(g().windows(100, 'only-full'));
assert.sameValue(result.length, 0, 'explicit "only-full" yields nothing');
