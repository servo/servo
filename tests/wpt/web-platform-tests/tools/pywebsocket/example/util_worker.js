// Copyright 2014 Google Inc. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the COPYING file or at
// https://developers.google.com/open-source/licenses/bsd

// Utilities for example applications (for the worker threads only).

onmessage = function (message) {
  var config = message.data.config;
  config.addToLog = function(text) {
      postMessage({type: 'addToLog', data: text}); };
  config.addToSummary = function(text) {
      postMessage({type: 'addToSummary', data: text}); };
  config.measureValue = function(value) {
      postMessage({type: 'measureValue', data: value}); };
  config.notifyAbort = function() { postMessage({type: 'notifyAbort'}); };

  doAction(config, false, message.data.type);
};
