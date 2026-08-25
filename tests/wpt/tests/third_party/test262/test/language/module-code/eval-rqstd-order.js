// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Requested modules are evaluated prior to the requesting module in source
    code order
esid: sec-moduleevaluation
info: |
    [...]
    6. For each String required that is an element of
       module.[[RequestedModules]] do,
       a. Let requiredModule be ? HostResolveImportedModule(module, required).
       b. Perform ? requiredModule.ModuleEvaluation().
    [...]
    16. Let result be the result of evaluating module.[[ECMAScriptCode]].
    [...]
includes: [fnGlobalObject.js]
flags: [module]
features: [export-star-as-namespace-from-module]
---*/

assert.sameValue(fnGlobalObject().test262, '123456789');

import {} from './eval-rqstd-order-1_FIXTURE.js';

import './eval-rqstd-order-2_FIXTURE.js';

import * as ns1 from './eval-rqstd-order-3_FIXTURE.js';

import dflt1 from './eval-rqstd-order-4_FIXTURE.js';

export {} from './eval-rqstd-order-5_FIXTURE.js';

import dflt2, {} from './eval-rqstd-order-6_FIXTURE.js';

export * from './eval-rqstd-order-7_FIXTURE.js';

import dflt3, * as ns2 from './eval-rqstd-order-8_FIXTURE.js';

export * as ns3 from './eval-rqstd-order-9_FIXTURE.js';
