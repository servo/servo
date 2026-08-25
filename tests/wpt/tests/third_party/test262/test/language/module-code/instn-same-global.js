// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Modules share the same global `this` value
esid: sec-moduledeclarationinstantiation
info: |
    [...]
    6. Let env be NewModuleEnvironment(realm.[[GlobalEnv]]).
    7. Set module.[[Environment]] to env.
    [...]
includes: [fnGlobalObject.js]
flags: [module]
---*/

import './instn-same-global-set_FIXTURE.js';

var global = fnGlobalObject();

assert.sameValue(global.test262, 262);
