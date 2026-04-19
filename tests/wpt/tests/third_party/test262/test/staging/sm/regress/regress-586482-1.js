/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  pending
esid: pending
---*/
var expect = true;
var actual;

var checkCaller = function(me) {
    var caller = arguments.callee.caller;
    var callerIsMethod = (caller === me['doThing']);
    actual = callerIsMethod;
};

var MyObj = function() {
};

MyObj.prototype.doThing = function() {
    checkCaller(this);
};

(new MyObj()).doThing();

assert.sameValue(expect, actual, "ok");
