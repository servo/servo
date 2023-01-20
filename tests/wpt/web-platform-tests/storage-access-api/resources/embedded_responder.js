// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
"use strict";

test_driver.set_test_context(window.top);

window.addEventListener("message", async (event) => {
  function reply(data) {
    event.source.postMessage(data, event.origin);
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
    case "document.cookie":
      reply(document.cookie);
      break;
    case "set_permission":
      await test_driver.set_permission(...event.data.args);
      reply(undefined);
      break;
    case "reload":
      window.location.reload();
      break;
    default:
  }
});
