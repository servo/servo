// META: title=XMLHttpRequest: navigation should not fire abort event

async_test(function(t) {
  var iframe = document.createElement("iframe");
  var events = [];

  window.addEventListener("message", t.step_func(function(e) {
    if (e.data.type === "xhr-event") {
      events.push(e.data.event);
    }
    if (e.data.type === "ready") {
      iframe.src = "/common/blank.html";
      iframe.onload = t.step_func(function() {
        t.step_timeout(t.step_func_done(function() {
          assert_false(events.includes("error"), "error event should not fire");
          assert_false(events.includes("abort"), "abort event should not fire");
          assert_false(events.includes("load"), "load event should not fire");
          assert_false(events.includes("upload.error"), "upload error event should not fire");
          assert_false(events.includes("upload.abort"), "upload abort event should not fire");
        }), 500);
      });
    }
  }));

  iframe.src = "/xhr/resources/abort-during-navigation-iframe.html";
  document.body.appendChild(iframe);
}, "XHR should not fire abort or error events when cancelled by navigation");
