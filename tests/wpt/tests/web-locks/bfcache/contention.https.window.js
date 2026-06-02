// META: title=Web Locks API: bfcache contention test
// META: timeout=long
// META: variant=?context=document&contention=request
// META: variant=?context=worker&contention=request
// META: variant=?context=nested-worker&contention=request
// META: variant=?context=shared-worker&contention=request
// META: variant=?context=document&contention=query
// META: variant=?context=worker&contention=query
// META: variant=?context=nested-worker&contention=query
// META: variant=?context=shared-worker&contention=query
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=/web-locks/resources/helpers.js
// META: script=/html/browsers/browsing-the-web/back-forward-cache/resources/helper.sub.js
// META: script=./helpers.js

const contexts = {
  'document': {
    description: 'the main thread',
    funcBeforeNavigation: (name) => {
      navigator.locks.request(name, () => new Promise(() => {}));
    },
  },
  'worker': {
    description: 'a worker',
    funcBeforeNavigation: async (name) => {
      window.worker = new Worker('/web-locks/resources/worker.js');
      await postToWorkerAndWait(worker, {op: 'request', name: name});
    },
  },
  'nested-worker': {
    description: 'a nested worker',
    funcBeforeNavigation: async (name) => {
      window.worker = new Worker('/web-locks/resources/parentworker.js');
      await postToWorkerAndWait(worker, {op: 'request', name: name});
    },
  },
  'shared-worker': {
    description: 'a shared worker',
    funcBeforeNavigation: async (name) => {
      window.worker = new SharedWorker('/web-locks/resources/worker.js');
      worker.port.start();
      await postToWorkerAndWait(worker.port, {op: 'request', name: name});
    },
  },
};

const contentions = {
  'request': {
    description: 'on navigator.locks.request()',
    funcBeforeBackNavigation: (name) => {
        return navigator.locks.request(name, () => {});
    },
  },
  'query': {
    description: 'on navigator.locks.query()',
    funcBeforeBackNavigation: () => navigator.locks.query(),
  }
};

const params = new URLSearchParams(location.search);
const context = contexts[params.get('context')];
const contention = contentions[params.get('contention')];
const lockName = uniqueNameByQuery();

runWebLocksBfcacheTest(
    {
      targetOrigin: originSameOrigin,
      argsBeforeNavigation: [lockName],
      argsBeforeBackNavigation: [lockName],
      funcBeforeNavigation: context.funcBeforeNavigation,
      funcBeforeBackNavigation: contention.funcBeforeBackNavigation,
      shouldBeCached: false,
    },
    `A held lock on ${context.description} should cause eviction ${
        contention.description}`);
