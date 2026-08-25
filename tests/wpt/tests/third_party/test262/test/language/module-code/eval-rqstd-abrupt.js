// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
    Abrupt completion during module evaluation precludes further evaluation
esid: sec-moduleevaluation
info: |
    [...]
    6. For each String required that is an element of
       module.[[RequestedModules]] do,
       a. Let requiredModule be ? HostResolveImportedModule(module, required).
       b. Perform ? requiredModule.ModuleEvaluation().
negative:
  phase: runtime
  type: TypeError
flags: [module]
---*/

import './eval-rqstd-abrupt-err-type_FIXTURE.js';
import './eval-rqstd-abrupt-err-uri_FIXTURE.js';

throw new RangeError();
