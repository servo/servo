// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
"use strict";

test_driver.set_test_context(window.top);

window.addEventListener("message", async (event) => {
  function reply(data) {
    event.source.postMessage(
        {timestamp: event.data.timestamp, data}, event.origin);
  }

  switch (event.data["command"]) {
    case "hasStorageAccess":
      reply(await document.hasStorageAccess());
      break;
    case "requestStorageAccess": {
      const obtainedAccess = await document.requestStorageAccess()
        .then(() => true, () => false);
      reply(obtainedAccess);
    }
      break;
    case "write document.cookie":
      document.cookie = event.data.cookie;
      reply(undefined);
      break;
    case "document.cookie":
      reply(document.cookie);
      break;
    case "set_permission":
      await test_driver.set_permission(...event.data.args);
      reply(undefined);
      break;
    case "observe_permission_change":
      const status = await navigator.permissions.query({name: "storage-access"});
      status.addEventListener("change", (event) => {
        reply(event.target.state)
      }, { once: true });
      break;
    case "reload":
      window.location.reload();
      break;
    case "httpCookies":
      // The `httpCookies` variable is defined/set by
      // script-with-cookie-header.py.
      reply(httpCookies);
      break;
    default:
  }
});
