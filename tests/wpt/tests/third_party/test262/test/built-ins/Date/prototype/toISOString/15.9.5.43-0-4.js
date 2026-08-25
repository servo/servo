// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
esid: sec-date.prototype.toisostring
description: >
    Date.prototype.toISOString - format of returned string is
    'YYYY-MM-DDTHH:mm:ss.sssZ', the time zone is UTC(0)
---*/

var date = new Date(1999, 9, 10, 10, 10, 10, 10);
var localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60000);

assert.sameValue(localDate.toISOString(), "1999-10-10T10:10:10.010Z", 'localDate.toISOString()');
