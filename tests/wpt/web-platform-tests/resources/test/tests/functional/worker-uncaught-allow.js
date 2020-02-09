importScripts("/resources/testharness.js");

setup({allow_uncaught_exception:true});

async_test(function(t) {
  onerror = function() {
    // Further delay the test's completion to ensure that the worker's
    // `onerror` handler does not influence results in the parent context.
    setTimeout(function() {
      t.done();
    }, 0);
  };

  setTimeout(function() {
    throw new Error("This error is expected.");
  }, 0);
}, 'onerror event is triggered');

done();
