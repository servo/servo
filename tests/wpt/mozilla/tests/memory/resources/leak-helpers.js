// Create an `<iframe>` element with the given script contents and then
// return true if `<iframe>`'s document is no longer resident in memory.
async function createAndDestroyIframeWithHTML(html) {
  const iframe = document.createElement("iframe");
  iframe.srcdoc = `<html>${html}<\/html>`;
  document.body.appendChild(iframe);
  // Wait for the document to load.
  await new Promise(resolve => iframe.onload = resolve);
  // A small timeout to ensure that things move forward enough.
  await new Promise(resolve => step_timeout(resolve, 100));

  const weakDocument = new WeakRef(iframe.contentDocument);
  iframe.remove();

  await TestUtils.gc();
  await TestUtils.gc();

  return weakDocument.deref() === undefined;
}
