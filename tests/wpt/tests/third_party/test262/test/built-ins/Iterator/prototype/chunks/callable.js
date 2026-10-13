// Copyright (C) 2026 Michael Ficarra. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.prototype.chunks
description: >
  Iterator.prototype.chunks is callable
features: [iterator-chunking, generators]
---*/
function* g() {}
Iterator.prototype.chunks.call(g(), 1);

let iter = g();
iter.chunks(1);
