// Copyright 2014 Cubane Canada, Inc.  All rights reserved.
// See LICENSE for details.

/*---
info: |
    Promise reaction jobs have predictable environment
es6id: S25.4.2.1_A2.1_T1
author: Sam Mikes
description: argument thrown through "Thrower"
flags: [async]
---*/

var obj = {};

var p = Promise.reject(obj).then( /*Identity, Thrower*/ )
  .then(function() {
    $DONE("Unexpected fulfillment - promise should reject.");
  }, function(arg) {
    if (arg !== obj) {
      $DONE("Expected reject reason to be obj, actually " + arg);
      return;
    }
    $DONE();
  });
