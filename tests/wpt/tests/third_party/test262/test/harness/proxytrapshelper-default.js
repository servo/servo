// Copyright (C) 2016 Jordan Harband. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
description: allowProxyTraps helper should default throw on all the proxy trap named methods being invoked
esid: pending
author: Jordan Harband
includes: [proxyTrapsHelper.js]
---*/

var traps = allowProxyTraps();

function assertTrapThrows(trap) {
    if (typeof traps[trap] !== 'function') {
        throw new Test262Error('trap ' + trap + ' is not a function');
    }
    var failedToThrow = false;
    try {
        traps[trap]();
        failedToThrow = true;
    } catch (e) {}
    if (failedToThrow) {
        throw new Test262Error('trap ' + trap + ' did not throw an error');
    }
}

assertTrapThrows('getPrototypeOf');
assertTrapThrows('setPrototypeOf');
assertTrapThrows('isExtensible');
assertTrapThrows('preventExtensions');
assertTrapThrows('getOwnPropertyDescriptor');
assertTrapThrows('has');
assertTrapThrows('get');
assertTrapThrows('set');
assertTrapThrows('deleteProperty');
assertTrapThrows('defineProperty');
assertTrapThrows('enumerate');
assertTrapThrows('ownKeys');
assertTrapThrows('apply');
assertTrapThrows('construct');
