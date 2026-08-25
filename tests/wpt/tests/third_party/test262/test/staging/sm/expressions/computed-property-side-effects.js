/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [sm/assertThrowsValue.js]
description: |
  Computed property names must be considered as always effectful even when the name expression isn't effectful, because calling ToPropertyKey on some non-effectful expressions has user-modifiable behavior
info: bugzilla.mozilla.org/show_bug.cgi?id=1199695
esid: pending
---*/

RegExp.prototype.toString = () => { throw 42; };
assertThrowsValue(function() {
  ({ [/regex/]: 0 }); // ToPropertyKey(/regex/) throws 42
}, 42);

function Q() {
  ({ [new.target]: 0 }); // new.target will be Q, ToPropertyKey(Q) throws 17
}
Q.toString = () => { throw 17; };
assertThrowsValue(function() {
  new Q;
}, 17);
