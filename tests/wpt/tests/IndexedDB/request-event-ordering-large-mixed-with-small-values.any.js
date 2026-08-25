// META: title=IndexedDB: request result events are delivered in order
// META: global=window,worker
// META: script=resources/support-promises.js
// META: script=resources/support.js
// META: script=resources/request-event-ordering-common.js
// META: timeout=long

// Spec: https://w3c.github.io/IndexedDB/#abort-transaction

'use strict';

eventsTest('large values mixed with small values', [
  ['get', 1],
  ['get', 2],
  ['get-empty', 5],
  ['count', 4],
  ['continue-empty', null],
  ['open', 1],
  ['continue', 2],
  ['open-empty', 5],
  ['getall', 4],
  ['open', 2],
  ['continue-empty', null],
  ['add', 5],
  ['get', 3],
  ['count', 5],
  ['get-empty', 6],
  ['put-with-id', 5],
  ['getall', 5],
  ['continue', 3],
  ['open-empty', 6],
  ['put', 6],
  ['error', 1],
  ['continue', 2],
  ['open', 4],
  ['get-empty', 7],
  ['count', 6],
  ['continue', 3],
  ['add', 7],
  ['getall', 7],
  ['error', 3],
  ['count', 7],
]);
