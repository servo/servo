/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
flags:
  - noStrict
description: |
  Attributes for properties of arguments objects
info: bugzilla.mozilla.org/show_bug.cgi?id=516255
esid: pending
---*/

// normal

function args() { return arguments; }
var a = args(0, 1);

var argProps = Object.getOwnPropertyNames(a).sort();
assert.sameValue(argProps.indexOf("callee") >= 0, true);
assert.sameValue(argProps.indexOf("0") >= 0, true);
assert.sameValue(argProps.indexOf("1") >= 0, true);
assert.sameValue(argProps.indexOf("length") >= 0, true);

var calleeDesc = Object.getOwnPropertyDescriptor(a, "callee");
assert.sameValue(calleeDesc.value, args);
assert.sameValue(calleeDesc.writable, true);
assert.sameValue(calleeDesc.enumerable, false);
assert.sameValue(calleeDesc.configurable, true);

var zeroDesc = Object.getOwnPropertyDescriptor(a, "0");
assert.sameValue(zeroDesc.value, 0);
assert.sameValue(zeroDesc.writable, true);
assert.sameValue(zeroDesc.enumerable, true);
assert.sameValue(zeroDesc.configurable, true);

var oneDesc = Object.getOwnPropertyDescriptor(a, "1");
assert.sameValue(oneDesc.value, 1);
assert.sameValue(oneDesc.writable, true);
assert.sameValue(oneDesc.enumerable, true);
assert.sameValue(oneDesc.configurable, true);

var lengthDesc = Object.getOwnPropertyDescriptor(a, "length");
assert.sameValue(lengthDesc.value, 2);
assert.sameValue(lengthDesc.writable, true);
assert.sameValue(lengthDesc.enumerable, false);
assert.sameValue(lengthDesc.configurable, true);


// strict

function strictArgs() { "use strict"; return arguments; }
var sa = strictArgs(0, 1);

var strictArgProps = Object.getOwnPropertyNames(sa).sort();
assert.sameValue(strictArgProps.indexOf("callee") >= 0, true);
assert.sameValue(strictArgProps.indexOf("caller") >= 0, false);
assert.sameValue(strictArgProps.indexOf("0") >= 0, true);
assert.sameValue(strictArgProps.indexOf("1") >= 0, true);
assert.sameValue(strictArgProps.indexOf("length") >= 0, true);

var strictCalleeDesc = Object.getOwnPropertyDescriptor(sa, "callee");
assert.sameValue(typeof strictCalleeDesc.get, "function");
assert.sameValue(typeof strictCalleeDesc.set, "function");
assert.sameValue(strictCalleeDesc.get, strictCalleeDesc.set);
assert.sameValue(strictCalleeDesc.enumerable, false);
assert.sameValue(strictCalleeDesc.configurable, false);

var strictCallerDesc = Object.getOwnPropertyDescriptor(sa, "caller");
assert.sameValue(strictCallerDesc, undefined);

var strictZeroDesc = Object.getOwnPropertyDescriptor(sa, "0");
assert.sameValue(strictZeroDesc.value, 0);
assert.sameValue(strictZeroDesc.writable, true);
assert.sameValue(strictZeroDesc.enumerable, true);
assert.sameValue(strictZeroDesc.configurable, true);

var strictOneDesc = Object.getOwnPropertyDescriptor(sa, "1");
assert.sameValue(strictOneDesc.value, 1);
assert.sameValue(strictOneDesc.writable, true);
assert.sameValue(strictOneDesc.enumerable, true);
assert.sameValue(strictOneDesc.configurable, true);

var strictLengthDesc = Object.getOwnPropertyDescriptor(sa, "length");
assert.sameValue(strictLengthDesc.value, 2);
assert.sameValue(strictLengthDesc.writable, true);
assert.sameValue(strictLengthDesc.enumerable, false);
assert.sameValue(strictLengthDesc.configurable, true);
