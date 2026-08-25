// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.flatMap
description: >
  Iterator.prototype.flatMap is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.flatMap.call(g(), () => []);

let iter = g();
iter.flatMap(() => []);
