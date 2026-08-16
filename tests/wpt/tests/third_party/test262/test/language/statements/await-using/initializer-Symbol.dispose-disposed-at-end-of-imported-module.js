// Copyright (C) 2026 Kevin Gibbons. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-source-text-module-record-execute-module
description: Initialized value is disposed at end of Module imported from other module (Symbol.dispose)
flags: [module, async]
features: [explicit-resource-management, top-level-await]
---*/

import { disposed, resource } from './initializer-Symbol.dispose-disposed-at-end-of-imported-module_FIXTURE.js';

assert(disposed, 'resource should be disposed once imported module evaluation finishes');
$DONE();
