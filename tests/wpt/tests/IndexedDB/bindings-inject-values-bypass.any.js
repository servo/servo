// META: global=window,worker
// META: title=IndexedDB: ES bindings - Inject a key into a value - Values bypass chain and setters
// META: script=resources/support-promises.js

'use_strict';

promise_test(async t => {
  const db = await createDatabase(t, db => {
    db.createObjectStore('store', {autoIncrement: true, keyPath: 'a.b.c'});
  });

  Object.prototype.a = {b: {c: 'on proto'}};
  t.add_cleanup(() => { delete Object.prototype.a; });

  const tx = db.transaction('store', 'readwrite', {durability: "relaxed"});
  tx.objectStore('store').put({});
  const result = await promiseForRequest(t, tx.objectStore('store').get(1));

  assert_true(result.hasOwnProperty('a'),
              'Result should have own-properties overriding prototype.');
  assert_true(result.a.hasOwnProperty('b'),
              'Result should have own-properties overriding prototype.');
  assert_true(result.a.b.hasOwnProperty('c'),
              'Result should have own-properties overriding prototype.');
  assert_equals(result.a.b.c, 1,
                'Own property should match primary key generator value');
  assert_equals(Object.prototype.a.b.c, 'on proto',
                'Prototype should not be modified');
}, 'Returning values to script should bypass prototype chain');

promise_test(async t => {
  const db = await createDatabase(t, db => {
    db.createObjectStore('store', {autoIncrement: true, keyPath: 'id'});
  });

  let setter_called = false;
  Object.defineProperty(Object.prototype, 'id', {
    configurable: true,
    set: value => { setter_called = true; },
  });
  t.add_cleanup(() => { delete Object.prototype['id']; });

  const tx = db.transaction('store', 'readwrite', {durability: 'relaxed'});
  tx.objectStore('store').put({});
  const result = await promiseForRequest(t, tx.objectStore('store').get(1));

  assert_false(setter_called,
               'Setter should not be called for key result.');
  assert_true(result.hasOwnProperty('id'),
              'Result should have own-property overriding prototype setter.');
  assert_equals(result.id, 1,
                'Own property should match primary key generator value');
}, 'Returning values to script should bypass prototype setters');
