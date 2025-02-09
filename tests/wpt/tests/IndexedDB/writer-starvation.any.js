// META: title=IndexedDB writer starvation test
// META: global=window,worker
// META: script=resources/support.js
// META: timeout=long

'use strict';

async_test(t => {
  let db;
  let read_request_count = 0;
  let read_success_count = 0;
  let write_request_count = 0;
  let write_success_count = 0;
  const RQ_COUNT = 25;

  const open_rq = createdb(t);
  open_rq.onupgradeneeded = t.step_func((e) => {
    db = e.target.result;
    db.createObjectStore('s').add('1', 1);
  });

  open_rq.onsuccess = t.step_func((e) => {
    let i = 0;

    // Pre-fill some read requests.
    for (i = 0; i < RQ_COUNT; i++) {
      read_request_count++;

      db.transaction('s', 'readonly').objectStore('s').get(1).onsuccess =
          t.step_func((e) => {
            read_success_count++;
            assert_equals(e.target.transaction.mode, 'readonly');
          });
    }

    t.step(loop);

    function loop() {
      read_request_count++;

      db.transaction('s', 'readonly').objectStore('s').get(1).onsuccess =
          t.step_func((e) => {
            read_success_count++;
            assert_equals(e.target.transaction.mode, 'readonly');

            if (read_success_count >= RQ_COUNT && write_request_count == 0) {
              write_request_count++;

              db.transaction('s', 'readwrite')
                  .objectStore('s')
                  .add('written', read_request_count)
                  .onsuccess = t.step_func((e) => {
                write_success_count++;
                assert_equals(e.target.transaction.mode, 'readwrite');
                assert_equals(
                    e.target.result, read_success_count,
                    'write cb came before later read cb\'s');
              });

              // Reads done after the write.
              for (i = 0; i < 5; i++) {
                read_request_count++;

                db.transaction('s', 'readonly')
                    .objectStore('s')
                    .get(1)
                    .onsuccess = t.step_func((e) => {
                  read_success_count++;
                });
              }
            }
          });

      if (read_success_count < RQ_COUNT + 5) {
        step_timeout(t.step_func(loop), write_request_count ? 1000 : 100);
      } else {
        // This runs finish() once `read_success_count` >= RQ_COUNT + 5.
        db.transaction('s', 'readonly').objectStore('s').count().onsuccess =
            t.step_func(() => {
              step_timeout(t.step_func(finish), 100);
            });
      }
    }
  });

  function finish() {
    assert_equals(read_request_count, read_success_count, 'read counts');
    assert_equals(write_request_count, write_success_count, 'write counts');
    t.done();
  }
}, 'IDB read requests should not starve write requests');
