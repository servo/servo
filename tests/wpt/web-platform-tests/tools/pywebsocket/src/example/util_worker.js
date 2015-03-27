// Copyright 2014 Google Inc. All rights reserved.
//
// Use of this source code is governed by a BSD-style
// license that can be found in the COPYING file or at
// https://developers.google.com/open-source/licenses/bsd

// Utilities for example applications (for the worker threads only).

function workerAddToLog(text) {
  postMessage({type: 'addToLog', data: text});
}

function workerAddToSummary(text) {
  postMessage({type: 'addToSummary', data: text});
}

function workerMeasureValue(value) {
  postMessage({type: 'measureValue', data: value});
}
