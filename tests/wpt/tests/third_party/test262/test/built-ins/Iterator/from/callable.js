// Copyright (C) 2020 Rick Waldron. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-iterator.from
description: >
  Iterator.from is callable
features: [iterator-helpers]
---*/
function* g() {}

Iterator.from(g());
Iterator.from.call(null, g());
