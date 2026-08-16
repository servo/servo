// Copyright (C) 2025 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-iterator.prototype.join
description: Iterator.prototype.join allows empty string as separator.
features: [Iterator.prototype.join]
---*/

assert.sameValue([].values().join(''), '');

assert.sameValue(['one'].values().join(''), 'one');

assert.sameValue(['one', 'two'].values().join(''), 'onetwo');

assert.sameValue(['one', 'two', 'three'].values().join(''), 'onetwothree');
