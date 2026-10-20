// Create an `<iframe>` element with the given script contents and then
// return true if `<iframe>`'s document is no longer resident in memory.
async function createAndDestroyIframeWithHTML(html) {
  const waitForFrameMessage = new Promise((resolve) => {
    window.addEventListener("message", (event) => {
      if (event.data == "testStarted") {
        resolve();
      }
    })
  });

  const iframe = document.createElement("iframe");
  iframe.srcdoc = `<html><script>window.parent.postMessage("testStarted");</script>${html}<\/html>`;
  document.body.appendChild(iframe);

  // Wait for the document to signal that it has parsed and started the test.
  await waitForFrameMessage;
  // A small timeout to ensure that things move forward enough.
  await new Promise(resolve => step_timeout(resolve, 100));

  const weakDocument = new WeakRef(iframe.contentDocument);
  iframe.remove();

  await TestUtils.gc();
  await TestUtils.gc();

  return weakDocument.deref() === undefined;
}
