// Copyright (C) 2025 Mozilla Foundation. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-create-text-module
description: Supports self-referential text imports
flags: [module]
features: [import-attributes, import-text]
---*/

import value from './text-self.js' with { type: 'text' };

assert.sameValue(typeof value, 'string');
