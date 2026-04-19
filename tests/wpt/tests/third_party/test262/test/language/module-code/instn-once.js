// Copyright (C) 2016 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: Module is linked exactly once
esid: sec-moduledeclarationlinking
info: |
  Link ( ) Concrete Method
    [...]
    4. Let result be InnerModuleLinking(module, stack, 0).
    [...]

  InnerModuleLinking( module, stack, index )
    [...]
    2. If module.[[Status]] is "linking", "linked", or "evaluated", then
      a. Return index.
    3. Assert: module.[[Status]] is "unlinked".
    4. Set module.[[Status]] to "linking".
    [...]
    9. For each String required that is an element of module.[[RequestedModules]], do
      a. Let requiredModule be ? HostResolveImportedModule(module, required).
      b. Set index to ? InnerModuleLinking(requiredModule, stack, index).
    [...]
flags: [module]
features: [export-star-as-namespace-from-module]
---*/

import {} from './instn-once.js';
import './instn-once.js';
import * as ns1 from './instn-once.js';
import dflt1 from './instn-once.js';
export {} from './instn-once.js';
import dflt2, {} from './instn-once.js';
export * from './instn-once.js';
export * as ns2 from './instn-once.js';
import dflt3, * as ns from './instn-once.js';
export default null;

let x;
