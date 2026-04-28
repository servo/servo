// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iteratorprototype.forEach
description: >
  Iterator.prototype.forEach is callable
features: [iterator-helpers]
---*/
function* g() {}
Iterator.prototype.forEach.call(g(), () => {});

let iter = g();
iter.forEach(() => {});
