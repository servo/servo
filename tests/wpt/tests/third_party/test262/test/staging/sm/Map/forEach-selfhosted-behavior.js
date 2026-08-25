/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
description: |
  Don't use .call(...) in the self-hosted Map.prototype.forEach
info: bugzilla.mozilla.org/show_bug.cgi?id=987243
esid: pending
---*/

var functionCall = Function.prototype.call;

function throwSyntaxError()
{
  throw new SyntaxError("Function.prototype.call incorrectly called");
}

function lalala() {}

Function.prototype.call = throwSyntaxError;

new Map().forEach(throwSyntaxError);
new Map([[1, 2]]).forEach(lalala);
new Map([[1, 2], [3, 4]]).forEach(lalala);

Function.prototype.call = function() { this.set(42, "fnord"); };

new Map().forEach(throwSyntaxError);
new Map([[1, 2]]).forEach(lalala);
new Map([[1, 2], [3, 4]]).forEach(lalala);

var callCount = 0;
Function.prototype.call = function() { callCount++; };

new Map().forEach(throwSyntaxError);
new Map([[1, 2]]).forEach(lalala);
new Map([[1, 2], [3, 4]]).forEach(lalala);

assert.sameValue(callCount, 0);
