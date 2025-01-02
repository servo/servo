async_test(t => {
  const frame = document.body.appendChild(document.createElement("iframe"));
  t.add_cleanup(() => frame.remove());

  frame.onload = t.step_func(() => {
    // Right now the doc of the iframe inside "frame" is still "fully-active".
    // Navigate parent away, making the child iframe's doc "active", not "fully-active".
    frame.contentWindow.location = "/common/blank.html";

    frame.onload = t.step_func(() => {
      // The child iframe's doc is "active", not "fully-active", and should not receive the storage notification.
      sessionStorage.setItem('myCat', 'Tom');
      t.step_timeout(() => {
        // The child iframe's hasn't received the storage notification.
        assert_equals(sessionStorage.getItem("Received storage event"), null);
        frame.contentWindow.history.go(-1);
        t.step_timeout(() => {
          // Now The child iframe's doc is "fully-active" again,
          // the previously not run storage task should now have been run.
          assert_equals(sessionStorage.getItem("Received storage event"), "true");
          t.done();
        }, 1000);
      }, 1000);
    });
  });

  frame.src = "resources/page-with-frame.html";
}, "Tasks for documents that are not fully active are stored, and run when the documents becomes fully-active");

