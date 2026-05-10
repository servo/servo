// Copyright (C) 2018 André Bargull. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-meta-properties-runtime-semantics-evaluation
description: >
  The same import.meta object is returned for a module.
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
    5. Else,
        a. Assert: Type(importMeta) is Object.
        b. Return importMeta.
flags: [module]
features: [import.meta]
---*/

var a = import.meta;
var b = function() { return import.meta; }();

assert.sameValue(import.meta, a,
                 "import.meta accessed directly and accessed via variable declaration");

assert.sameValue(import.meta, b,
                 "import.meta accessed directly and accessed via function return value");
