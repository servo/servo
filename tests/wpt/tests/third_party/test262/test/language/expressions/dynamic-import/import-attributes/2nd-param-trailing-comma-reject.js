// Copyright (C) 2021 the V8 project authors. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.
/*---
description: >
  ImportCall parameter list supports an optional trailing comma (rejection
  semantics)
esid: sec-import-call-runtime-semantics-evaluation
info: |
  ImportCall[Yield, Await]:
    import ( AssignmentExpression[+In, ?Yield, ?Await] ,opt )
    import ( AssignmentExpression[+In, ?Yield, ?Await] , AssignmentExpression[+In, ?Yield, ?Await] ,opt )
features: [dynamic-import, import-attributes]
flags: [async]
---*/

var thrown = new Test262Error();

import({toString: function() { throw thrown; } }, {},)
  .then(function() {
    throw new Test262Error('Expected promise to be rejected, but it was fulfilled.');
  }, function(caught) {
    assert.sameValue(thrown, caught);
  })
  .then($DONE, $DONE);
