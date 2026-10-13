// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  The value returned by Iterator.prototype.chunks is an Iterator instance
info: |
  Iterator.prototype.chunks ( chunkSize )

  6. Let result be CreateIteratorFromClosure(closure, "Iterator Helper", %IteratorHelperPrototype%, « [[UnderlyingIterators]] »).

features: [iterator-chunking, generators]
---*/

assert(
  (function* () {})().chunks(1) instanceof Iterator,
  'function*(){}().chunks(1) must return an Iterator'
);
