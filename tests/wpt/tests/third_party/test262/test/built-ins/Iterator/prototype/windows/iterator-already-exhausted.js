// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  Iterator.prototype.windows yields no windows when the iterator is already
  exhausted, regardless of undersized mode
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  8.a.i. Let value be ? IteratorStepValue(iterated).
  8.a.ii. If value is ~done~, then
    8.a.ii.a. If undersized is "allow-partial", buffer is not empty, and ...
    8.a.ii.b. Return ReturnCompletion(undefined).

features: [iterator-chunking, generators]
---*/
function* g() {}

let windows = Array.from(g().windows(2));
assert.sameValue(windows.length, 0, 'default undersized on empty iterator');

windows = Array.from(g().windows(2, 'only-full'));
assert.sameValue(windows.length, 0, '"only-full" on empty iterator');

windows = Array.from(g().windows(2, 'allow-partial'));
assert.sameValue(windows.length, 0, '"allow-partial" on empty iterator');
