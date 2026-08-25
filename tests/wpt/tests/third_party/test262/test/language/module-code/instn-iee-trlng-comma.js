// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    ExportsList in ExportDeclaration may include a trailing comma
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    9. For each ExportEntry Record e in module.[[IndirectExportEntries]], do
       a. Let resolution be ? module.ResolveExport(e.[[ExportName]], « », « »).
       [...]
flags: [module]
---*/

export { a , } from './instn-iee-trlng-comma_FIXTURE.js';
export { a as b , } from './instn-iee-trlng-comma_FIXTURE.js';

import { a, b } from './instn-iee-trlng-comma.js';

assert.sameValue(a, 333, 'comma following named export');
assert.sameValue(b, 333, 'comma following re-named export');
