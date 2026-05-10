// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.some
description: >
  Iterator.prototype.some is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.some.call(g(), () => {});

let iter = g();
iter.some(() => {});
