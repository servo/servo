'use strict';

function worker_test(file) {
  fetch_tests_from_worker(new Worker(file));
  if (typeof SharedWorker === 'function') {
    fetch_tests_from_worker(new SharedWorker(file));
  } else {
    test(() => {
      assert_unreached('SharedWorker is unavailable');
    }, 'Load ' + file + ' with SharedWorker');
  }
  service_worker_test(file);
}

