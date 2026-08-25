// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-meta-properties-runtime-semantics-evaluation
description: >
  The import.meta object is not shared across modules.
info: |
  Runtime Semantics: Evaluation

   ImportMeta : import.meta

    1. Let module be GetActiveScriptOrModule().
    ...
    3. Let importMeta be module.[[ImportMeta]].
    4. If importMeta is undefined.
        ...
        f. Set module.[[ImportMeta]] to importMeta.
        g. Return importMeta.
    ...
flags: [module]
features: [import.meta]
---*/

import {meta as fixture_meta, getMeta} from "./distinct-for-each-module_FIXTURE.js";

// The imported module has a distinct import.meta object.
assert.notSameValue(import.meta, fixture_meta,
                    "foreign import.meta accessed via import binding");
assert.notSameValue(import.meta, getMeta(),
                    "foreign import.meta accessed via function call");

// Calling a function which returns import.meta returns the import.meta object
// from the module in which the function is declared.
assert.sameValue(fixture_meta, getMeta(),
                 "import.meta accessed via import binding is identical to the one accessed via call");
