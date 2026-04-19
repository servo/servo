// Copyright (C) 2018 Leo Balter. All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

var startTime = Date.now();
var endTime;

export { endTime as time }

while (true) {
    endTime = Date.now() - startTime;
    if (endTime > 100) {
        break;
    }
}
