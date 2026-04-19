// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
   Promise.resolve
es6id: S25.4.4.5
author: Sam Mikes
description: Promise.resolve delegates to foreign thenable
flags: [async]
---*/

var thenable = {
  then: function(onResolve, onReject) {
    return onResolve('resolved');
  }
};

var p = Promise.resolve(thenable);

p.then(function(r) {
  assert.sameValue(r, 'resolved');
}).then($DONE, $DONE);
