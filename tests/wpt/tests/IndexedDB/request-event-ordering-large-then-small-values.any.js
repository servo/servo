// META: title=IndexedDB: request result events are delivered in order
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/support.js
// META: script=resources/request-event-ordering-common.js
// META: timeout=long

// Spec: https://w3c.github.io/IndexedDB/#abort-transaction

'use strict';

eventsTest('large value followed by small values', [
  ['get', 1],
  ['getall', 4],
  ['open', 2],
  ['continue-empty', null],
  ['get', 2],
  ['get-empty', 5],
  ['count', 4],
  ['continue-empty', null],
  ['open-empty', 5],
  ['add', 5],
  ['error', 1],
  ['continue', 2],
  ['get-empty', 6],
  ['put-with-id', 5],
  ['put', 6],
]);
