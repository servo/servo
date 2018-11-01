// META: script=support-promises.js

promise_test(async testCase => {
  // Delete any databases that may not have been cleaned up after
  // previous test runs.
  await deleteAllDatabases(testCase);

  const db_name = "TestDatabase";
  const db = await createNamedDatabase(testCase, db_name, ()=>{});
  const databases_promise = await indexedDB.databases();
  const expected_result = [
    {"name": db_name, "version": 1},
  ];
  assert_object_equals(
      databases_promise,
      expected_result,
      "Call to databases() did not retrieve the single expected result.");
}, "Enumerate one database.");

promise_test(async testCase => {
  // Delete any databases that may not have been cleaned up after previous test
  // runs.
  await deleteAllDatabases(testCase);

  const db_name1 = "TestDatabase1";
  const db_name2 = "TestDatabase2";
  const db_name3 = "TestDatabase3";
  const db1 = await createNamedDatabase(testCase, db_name1, ()=>{});
  const db2 = await createNamedDatabase(testCase, db_name2, ()=>{});
  const db3 = await createNamedDatabase(testCase, db_name3, ()=>{});
  const databases_promise = await indexedDB.databases();
  const expected_result = [
    {"name": db_name1, "version": 1},
    {"name": db_name2, "version": 1},
    {"name": db_name3, "version": 1},
  ];
  assert_object_equals(
    databases_promise,
    expected_result,
    "Call to databases() did not retrieve the multiple expected results");
}, "Enumerate multiple databases.");

promise_test(async testCase => {
  // Add some databases and close their connections.
  const db1 = await createNamedDatabase(testCase, "DB1", ()=>{});
  const db2 = await createNamedDatabase(testCase, "DB2", ()=>{});
  db1.onversionchange = () => { db1.close() };
  db2.onversionchange = () => { db2.close() };

  // Delete any databases that may not have been cleaned up after previous test
  // runs as well as the two databases made above.
  await deleteAllDatabases(testCase);

  // Make sure the databases are no longer returned.
  const databases_promise = await indexedDB.databases();
  assert_object_equals(
    databases_promise,
    [],
    "Call to databases() found database it should not have.")
}, "Make sure an empty list is returned for the case of no databases.");

done();
