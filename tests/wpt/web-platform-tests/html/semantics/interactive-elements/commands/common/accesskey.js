setup({explicit_done: true, explicit_timeout: true});

const NOTRUN = 3;
let status = NOTRUN;
function notrun() {
  return status === NOTRUN;
}
add_completion_callback(tests => {
  status = tests[0].status;
});

function pass() {
  // Wait a couple of frames in case fail() is also called.
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      if (notrun()) {
        test(() => {});
        done();
      }
    });
  });
}

function fail(msg) {
  if (notrun()) {
    test(() => { assert_unreached(msg); });
    done();
  }
}

document.addEventListener('DOMContentLoaded', () => {
  const accessKeyElement = document.querySelector('[accesskey]');
  if (accessKeyElement.accessKeyLabel) {
    document.querySelector('kbd').textContent = accessKeyElement.accessKeyLabel;
  }
});
