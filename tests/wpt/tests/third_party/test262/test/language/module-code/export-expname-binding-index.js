// Copyright (C) 2021 Alexey Shvayka. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-module-namespace-exotic-objects
description: >
  Internal methods of module namespace exotic objects are correct
  with non-Identifier bindings that are integer indices.
info: |
  [[HasProperty]] ( P )

  [...]
  3. If P is an element of exports, return true.
  4. Return false.

  [[Get]] ( P, Receiver )

  [...]
  13. Return ? targetEnv.GetBindingValue(binding.[[BindingName]], true).

  [[Set]] ( P, V, Receiver )

  1. Return false.

  [[Delete]] ( P )

  [...]
  4. If P is an element of exports, return false.
  5. Return true.
flags: [module]
features: [arbitrary-module-namespace-names, Reflect]
---*/
import * as ns from "./export-expname-binding-index_FIXTURE.js";

assert.sameValue(ns[0], 0);
assert.sameValue(Reflect.get(ns, 1), 1);
assert.sameValue(ns[2], undefined);

assert.throws(TypeError, () => { ns[0] = 1; });
assert(!Reflect.set(ns, 1, 1));
assert.throws(TypeError, () => { ns[2] = 2; });

assert(0 in ns);
assert(Reflect.has(ns, 1));
assert(!(2 in ns));

assert.throws(TypeError, () => { delete ns[0]; });
assert(!Reflect.deleteProperty(ns, 1));
assert(delete ns[2]);
