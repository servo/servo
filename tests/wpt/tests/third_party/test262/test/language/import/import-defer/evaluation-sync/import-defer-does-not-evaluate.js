// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-innermoduleevaluation
description: >
  `import defer` by itself does not trigger evaluation of sync modules
info: |
  16.2.1.5.3.1 InnerModuleEvaluation ( _module_, _stack_, _index_ )
    1. ...
    1. Let _evaluationList_ be a new empty List.
    1. For each ModuleRequest Record _required_ of _module_.[[RequestedModules]], do
        1. Let _requiredModule_ be GetImportedModule(_module_, _required_.[[Specifier]]).
        1. If _required_.[[Phase]] is ~defer~, then
            i. Let _additionalModules_ be GatherAsynchronousTransitiveDependencies(_requiredModule_).
            ii. For each Module Record _additionalModule_ of _additionalModules_, do
                1. If _evaluationList_ does not contain _additionalModule_, then
                    a. Append _additionalModule_ to _evaluationList_.
        1. Else if _evaluationList_ does not contain _requiredModule_, then
            i. Append _requiredModule_ to _evaluationList_.
    1. ...
    1. For each Module Record _requiredModule_ of _evaluationList_, do
      1. Set _index_ to ? InnerModuleEvaluation(_requiredModule_, _stack_, _index_).
      1. ...

flags: [module]
features: [import-defer]
includes: [compareArray.js]
---*/

import "./setup_FIXTURE.js";

import defer * as ns1 from "./dep-1_FIXTURE.js";

assert.sameValue(globalThis.evaluations.length, 0, "import defer does not trigger evaluation");

const ns_1_2 = ns1.ns_1_2;

assert.compareArray(globalThis.evaluations, [1.1, 1], "when evaluation is triggered, deferred sub-dependencies are not evaluated");

ns1.ns_1_2;

assert.compareArray(globalThis.evaluations, [1.1, 1], "the module is not re-executed");

ns_1_2.foo;

assert.compareArray(globalThis.evaluations, [1.1, 1, 1.2]);
