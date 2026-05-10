// Copyright (C) 2018 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: Succeeds when an entry object is a boxed string.
esid: sec-object.fromentries
features: [Object.fromEntries]
---*/

var result = Object.fromEntries([Object('ab')]);
assert.sameValue(result['a'], 'b');
