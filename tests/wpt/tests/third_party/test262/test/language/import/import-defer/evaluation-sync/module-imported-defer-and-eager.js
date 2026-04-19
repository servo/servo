// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-static-semantics-modulerequests
description: >
  `import defer` by itself does not trigger evaluation of sync modules
info: |
  Static Semantics: ModuleRequests
    ModuleItemList : ModuleItemList ModuleItem
      1. Let _requests_ be ModuleRequests of ModuleItemList.
      1. Let _additionalRequests_ be ModuleRequests of ModuleItem.
      1. For each ModuleRequest Record _mr_ of _additionalRequests_, do
          1. Let _found_ be false.
          1. For each ModuleRequest Record _mr2_ of _requests_, do
              1. If _mr_.[[Specifier]] is _mr2_.[[Specifier]] and _mr_.[[Phase]] is _mr2_.[[Phase]], then
                  1. Assert: _found_ is false.
                  1. Set _found_ to true.
          1. If _found_ is false, then
              1. Append _mr_ to _requests_.
      1. Return _requests_.

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
import "./dep-2_FIXTURE.js";
import "./dep-1_FIXTURE.js";

assert.compareArray(globalThis.evaluations, [2, 1.1, 1], "the module is evaluated in the order where it's imported as non-deferred");
