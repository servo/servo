// Copyright (C) 2024 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.concat
description: >
  Iterator.concat when called with zero arguments.
info: |
  Iterator.concat ( ...items )

  1. Let iterables be a new empty List.
  2. For each element item of items, do
    ...
  3. Let closure be a new Abstract Closure with no parameters that captures iterables and performs the following steps when called:
    a. For each Record iterable of iterables, do
      ...
    b. Return ReturnCompletion(undefined).
  ...
  6. Return gen.
features: [iterator-sequencing]
---*/

let iterator = Iterator.concat();

let iterResult = iterator.next();

assert.sameValue(iterResult.done, true);
assert.sameValue(iterResult.value, undefined);
