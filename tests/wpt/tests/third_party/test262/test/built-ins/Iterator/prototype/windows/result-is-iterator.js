// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.windows
description: >
  The value returned by Iterator.prototype.windows is an Iterator instance
info: |
  Iterator.prototype.windows ( windowSize [ , undersized ] )

  8. Let result be CreateIteratorFromClosure(closure, "Iterator Helper", %IteratorHelperPrototype%, « [[UnderlyingIterators]] »).

features: [iterator-chunking, generators]
---*/

assert(
  (function* () {})().windows(1) instanceof Iterator,
  'function*(){}().windows(1) must return an Iterator'
);
