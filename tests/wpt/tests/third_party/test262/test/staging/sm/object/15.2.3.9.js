/*
 * Any copyright is dedicated to the Public Domain.
 * http://creativecommons.org/licenses/publicdomain/
 */

/*---
includes: [propertyHelper.js]
description: |
  pending
esid: pending
---*/
/* Object.freeze */

function getme() { return 42; };
function setme(x) { };

var properties = { all:       { value:1, writable:true,  configurable:true,  enumerable: true },
                   readOnly:  { value:2, writable:false, configurable:true,  enumerable: true },
                   nonConfig: { value:3, writable:true,  configurable:false, enumerable: true },
                   none:      { value:4, writable:false, configurable:false, enumerable: true },
                   getter:    { get: getme,              configurable:false, enumerable: true },
                   setter:    { set: setme,              configurable:false, enumerable: true },
                   getandset: { get: getme, set: setme,  configurable:false, enumerable: true }
                 };
var o = Object.defineProperties({}, properties);

Object.freeze(o);

verifyProperty(o, "all",       { value: 1, writable: false, enumerable: true, configurable: false });
verifyProperty(o, "readOnly",  { value: 2, writable: false, enumerable: true, configurable: false });
verifyProperty(o, "nonConfig", { value: 3, writable: false, enumerable: true, configurable: false });
verifyProperty(o, "none",      { value: 4, writable: false, enumerable: true, configurable: false });
verifyProperty(o, "getter",    { get: getme, set: (void 0), enumerable: true, configurable: false });
verifyProperty(o, "setter",    { set: setme, get: (void 0), enumerable: true, configurable: false });
verifyProperty(o, "getandset", { get: getme, set: setme,    enumerable: true, configurable: false });
