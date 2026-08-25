// META: title=RemoteContextHelper createContext with throwing/rejecting executorCreators.
// META: script=/common/dispatcher/dispatcher.js
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/html/browsers/browsing-the-web/remote-context-helper/resources/remote-context-helper.js

'use strict';

promise_test(async t => {
  const rcHelper = new RemoteContextHelper();

  const err = new Error('something bad!');
  promise_rejects_exactly(
      t, err, rcHelper.createContext({ executorCreator() { throw err; } }),
      'Sync exception must be rethrown');

  promise_rejects_exactly(
      t, err, rcHelper.createContext({ executorCreator() { return Promise.reject(err); } }),
      'Async rejection must be rethrown');
});
