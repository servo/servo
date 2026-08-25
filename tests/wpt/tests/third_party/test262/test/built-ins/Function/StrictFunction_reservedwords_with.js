// Copyright (C) 2020 Xing Qu. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
esid: sec-createdynamicfunction
description: Strictfunction shouldn't have the reserved word "with"
info: |
    CreateDynamicFunction ( constructor, newTarget, kind, args )
    
    ...
    20 Perform the following substeps in an implementation-dependent order, possibly interleaving parsing and error detection:
      ...
      c. Let strict be ContainsUseStrict of body.
      d. If any static semantics errors are detected for parameters or body, throw a SyntaxError exception. 
         If strict is true, the Early Error rules for UniqueFormalParameters:FormalParameters are applied.
      ...
    ...
---*/

assert.throws(SyntaxError, function() {
  new Function("'use strict'; with ({}) {}");
}, '`new Function("\'use strict\'; with ({}) {}")` throws a SyntaxError exception');
