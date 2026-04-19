// Copyright (c) 2012 Ecma International.  All rights reserved.
// This code is governed by the BSD license found in the LICENSE file.

/*---
es5id: 15.2.3.6-4-588
description: >
    ES5 Attributes - [[Get]] field of inherited property of
    [[Prototype]] internal property is correct (Object.create)
---*/

var appointment = {};

var data1 = 1001;
Object.defineProperty(appointment, "startTime", {
  get: function() {
    return data1;
  },
  enumerable: true,
  configurable: false
});
var data2 = "NAME";
Object.defineProperty(appointment, "name", {
  get: function() {
    return data2;
  },
  set: function(value) {
    data2 = value;
  },
  enumerable: true,
  configurable: true
});

var meeting = Object.create(appointment);
var data3 = "In-person meeting";
Object.defineProperty(meeting, "conferenceCall", {
  get: function() {
    return data3;
  },
  enumerable: true,
  configurable: false
});

var teamMeeting = Object.create(meeting);

var hasOwnProperty = !teamMeeting.hasOwnProperty("name") &&
  !teamMeeting.hasOwnProperty("startTime") &&
  !teamMeeting.hasOwnProperty('conferenceCall');

assert(hasOwnProperty, 'hasOwnProperty !== true');
assert.sameValue(teamMeeting.name, "NAME", 'teamMeeting.name');
assert.sameValue(teamMeeting.startTime, 1001, 'teamMeeting.startTime');
assert.sameValue(teamMeeting.conferenceCall, "In-person meeting", 'teamMeeting.conferenceCall');
