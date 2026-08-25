// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise reaction jobs have predictable environment
es6id: S25.4.2.1_A1.1_T1
author: Sam Mikes
description: argument passes through "Identity"
flags: [async]
---*/

var obj = {};

var p = Promise.resolve(obj).then( /*Identity, Thrower*/ )
  .then(function(arg) {
    if (arg !== obj) {
      $DONE("Expected promise to be fulfilled with obj, actually " + arg);
      return;
    }
    $DONE();
  }, function() {
    $DONE('The promise should not be rejected.');
  });
