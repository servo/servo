// META: title=Worker Termination Aborts a Pending Upgrade
// META: script=resources/support-promises.js

// This test verifies that if a Worker's shutdown races an IndexedDB
// versionchange transaction that is creating a database that the next attempt
// to open the database results in a versionchange from version 0 and that
// nothing was in the database.
//
// Care has been taken to make this test's behavior well-defined relative to the
// spec to avoid intermittent failures.  In particular
// `DedicatedWorkerGlobalScope.close()` is used on the worker after issuing the
// `IDBFactory.open()` call.  This precludes any further tasks running on the
// worker by spec, although implementations may potentially have "zones of
// danger" in the time between the worker transitioning and when any state
// machines on the parent thread realize what's going on.

async function runAsyncFunctionInWorkerThenClose(funcToStringify) {
  const script = `// This script was created by runAsyncFunctionInWorkerThenClose
let testFunc = ${funcToStringify.toString()};
setTimeout(async () => {
  await testFunc();
  postMessage("ran");
  self.close();
}, 0);
`;
  const scriptBlob = new Blob([script]);
  const url = URL.createObjectURL(scriptBlob);
  const w = new Worker(url);
  await new Promise((resolve) => {
    w.onmessage = (evt) => {
      if (evt.data === "ran") {
        resolve();
      }
    };
  });
  URL.revokeObjectURL(url);
}

promise_test(async t => {
  await runAsyncFunctionInWorkerThenClose(async function() {
    // Note that this code will actually run on the worker, so anything
    // lexically captured will be coming from the worker's global scope.
    const openReq = indexedDB.open("aborted-upgrade-db", 1);

    openReq.onupgradeneeded = (event) => {
      const db = event.target.result;
      db.createObjectStore("should-not-be-created");
    }
  });

  // At this point we know that the open request was issued on the worker
  // worker thread.  An ordering concern at this point is that IDB only
  // specifies that the the connection opening algorithm is run in parallel and
  // we are not guaranteed that when we go "in parallel" here that our operation
  // won't run first.  As such, it may be necessary to add some kind of
  // arbitrary delay in the future if implementations do not effectively
  // maintain sequential ordering of IPC requests within a process.
  //
  // Note that we must NOT use `createNamedDatabase` here because it will
  // issue a blind call to `deleteDatabase`.  Because the migrate helper does
  // not perform cleanup, we must add the cleanup deletion now, though.
  t.add_cleanup(() => { indexedDB.deleteDatabase("aborted-upgrade-db"); });
  let createdDB = await migrateNamedDatabase(t, "aborted-upgrade-db", 1, (db) => {
    assert_equals(db.objectStoreNames.length, 0, "DB should have been empty");
    // Let's make sure the database is not permanently broken / corrupted.
    db.createObjectStore("should-be-created");
  });

  assert_equals(createdDB.objectStoreNames.length, 1, "created object store correctly");
  assert_equals(createdDB.objectStoreNames.item(0), "should-be-created");
});
