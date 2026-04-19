// Copyright (C) 2024 Igalia, S.L. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-getmodulenamespace
description: >
  Deferred namespace objects are created and cached appropriately
info: |
  GetModuleNamespace ( _module_, _phase_ )
    1. ...
    1. If _phase_ is ~defer~, let _namespace_ be _module_.[[DeferredNamespace]], otherwise let _namespace_ be _module_.[[Namespace]].
    1. If _namespace_ is ~empty~, then
      1. ...
      1. Set _namespace_ to ModuleNamespaceCreate(_module_, _unambiguousNames_, _phase_).
    1. Return _namespace_.

  ModuleNamespaceCreate ( _module_, _exports_, _phase_ )
    1. ...
    1. Let _M_ be MakeBasicObject(_internalSlotsList_).
    1. ...
    1. If _phase_ is ~defer~, then
      1. Set _module_.[[DeferredNamespace]] to _M_.
      1. ...
    1. Else,
      1. Set _module_.[[Namespace]] to _M_.
      1. ...
    1. Return _M_.

flags: [module]
features: [import-defer]
---*/

import * as nsEager from "./dep_FIXTURE.js";

import defer * as nsDeferred1 from "./dep_FIXTURE.js";
import defer * as nsDeferred2 from "./dep_FIXTURE.js";
import { depDeferredNamespace as nsDeferred3 } from "./dep-defer-ns_FIXTURE.js";
const nsDeferred4 = await import.defer("./dep_FIXTURE.js");

assert.sameValue(nsDeferred1, nsDeferred2, "Deferred import of the same module twice gives the same object");
assert.sameValue(nsDeferred1, nsDeferred3, "Deferred import of the same module twice from different files gives the same object");
assert.sameValue(nsDeferred1, nsDeferred4, "Static and dynamic deferred import of the same module gives the same object");
assert.notSameValue(nsDeferred1, nsEager, "Deferred namespaces are distinct from eager namespaces");
