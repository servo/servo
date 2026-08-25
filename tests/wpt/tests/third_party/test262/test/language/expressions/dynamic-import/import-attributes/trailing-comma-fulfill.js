// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ImportCall parameter list supports an optional trailing comma (fulfillment
  semantics)
esid: sec-import-call-runtime-semantics-evaluation
info: |
  ImportCall[Yield, Await]:
    import ( AssignmentExpression[+In, ?Yield, ?Await] ,opt )
    import ( AssignmentExpression[+In, ?Yield, ?Await] , AssignmentExpression[+In, ?Yield, ?Await] ,opt )
features: [dynamic-import, import-attributes]
flags: [async]
---*/

import('./2nd-param_FIXTURE.js',)
  .then(function(module) {
    assert.sameValue(module.default, 262);
  })
  .then($DONE, $DONE);
