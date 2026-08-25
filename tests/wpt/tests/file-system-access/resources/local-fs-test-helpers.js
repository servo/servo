// This file defines a directory_test() function that can be used to define
// tests that require a FileSystemDirectoryHandle. The implementation of that
// function in this file will ask the user to select an empty directory and uses
// that directory.
//
// Another implementation of this function exists in
// fs/resources/sandboxed-fs-test-helpers.js, where that version uses the
// sandboxed file system instead.

function getFileSystemType() {
  return 'local';
}

const directory_promise = (async () => {
  await new Promise(resolve => {
    window.addEventListener('DOMContentLoaded', resolve);
  });

  // Small delay to give chrome's test automation a chance to actually install
  // itself.
  await new Promise(resolve => step_timeout(resolve, 100));

  await window.test_driver.bless(
      'show a file picker.<br />Please select an empty directory');
  const entries = await self.showDirectoryPicker();
  assert_true(entries instanceof FileSystemHandle);
  assert_true(entries instanceof FileSystemDirectoryHandle);
  for await (const entry of entries) {
    assert_unreached('Selected directory is not empty');
  }
  return entries;
})();

async function cleanupDirectory(dir, ignoreRejections) {
  // Get a snapshot of the entries.
  const entries = await Array.fromAsync(dir.values());

  // Call removeEntry on all of them.
  const remove_entry_promises = entries.map(
      entry =>
          dir.removeEntry(entry.name, {recursive: entry.kind === 'directory'}));

  // Wait for them all to resolve or reject.
  if (ignoreRejections) {
    await Promise.allSettled(remove_entry_promises);
  } else {
    await Promise.all(remove_entry_promises);
  }
}

function directory_test(func, description) {
  promise_test(async t => {
    const directory = await directory_promise;

    // To be extra resilient against bad tests, cleanup before every test.
    await cleanupDirectory(directory, /*ignoreRejections=*/ false);

    // Cleanup after every test.
    t.add_cleanup(async () => {
      // Ignore any rejections since other cleanup code may have deleted them
      // before we could.
      await cleanupDirectory(directory, /*ignoreRejections=*/ true);
    });

    await func(t, directory);
  }, description);
}

directory_test(async (t, dir) => {
  assert_equals(await dir.queryPermission({mode: 'read'}), 'granted');
}, 'User succesfully selected an empty directory.');

directory_test(async (t, dir) => {
  const status = await dir.queryPermission({mode: 'readwrite'});
  if (status == 'granted')
    return;

  await window.test_driver.bless('ask for write permission');
  assert_equals(await dir.requestPermission({mode: 'readwrite'}), 'granted');
}, 'User granted write access.');

const child_frame_js = (origin, frameFn, done) => `
  const importScript = ${importScript};
  await importScript("/html/cross-origin-embedder-policy/credentialless" +
                  "/resources/common.js");
  await importScript("/html/anonymous-iframe/resources/common.js");
  await importScript("/common/utils.js");
  await send("${done}", ${frameFn}("${origin}"));
`;

/**
 * Context identifiers for executor subframes of framed tests. Individual
 * contexts (or convenience context lists below) can be used to send JavaScript
 * for evaluation in each frame (see framed_test below).
 *
 * Note that within framed tests:
 *  - firstParty represents the top-level document.
 *  - thirdParty represents an embedded context (iframe).
 *  - ancestorBit contexts include a cross-site ancestor iframe.
 *  - anonymousFrame contexts are third-party anonymous iframe contexts.
 */
const FRAME_CONTEXT = {
  firstParty: 0,
  thirdPartySameSite: 1,
  thirdPartySameSite_AncestorBit: 2,
  thirdPartyCrossSite: 3,
  anonymousFrameSameSite: 4,
  anonymousFrameSameSite_AncestorBit: 5,
  anonymousFrameCrossSite: 6,
};

// TODO(crbug.com/1322897): Add AncestorBit contexts.
const sameSiteContexts = [
  FRAME_CONTEXT.firstParty,
  FRAME_CONTEXT.thirdPartySameSite,
  FRAME_CONTEXT.anonymousFrameSameSite,
];

// TODO(crbug.com/1322897): Add AncestorBit contexts.
const crossSiteContexts = [
  FRAME_CONTEXT.thirdPartyCrossSite,
  FRAME_CONTEXT.anonymousFrameCrossSite,
];

// TODO(crbug.com/1322897): Add AncestorBit contexts.
const childContexts = [
  FRAME_CONTEXT.thirdPartySameSite,
  FRAME_CONTEXT.thirdPartyCrossSite,
  FRAME_CONTEXT.anonymousFrameSameSite,
  FRAME_CONTEXT.anonymousFrameCrossSite,
];

/**
 * Creates a promise test with same- & cross-site executor subframes.
 *
 * In addition to the standard testing object, the provided func will be called
 * with a sendTo function. sendTo expects:
 *   - contexts: an Iterable of FRAME_CONTEXT constants representing the
 *               frame(s) in which the provided script will be concurrently run.
 *   - js_gen: a function which should generate a script string when called
 *             with a string token. sendTo will wait until a "done" message
 *             is sent to this queue.
 */
function framed_test(func, description) {
  const same_site_origin = get_host_info().HTTPS_ORIGIN;
  const cross_site_origin = get_host_info().HTTPS_NOTSAMESITE_ORIGIN;
  const frames = Object.values(FRAME_CONTEXT);

  promise_test(async (t) => {
    return new Promise(async (resolve, reject) => {
      try {
        // Set up handles to all third party frames.
        const handles = [
          null,                          // firstParty
          newIframe(same_site_origin),   // thirdPartySameSite
          null,                          // thirdPartySameSite_AncestorBit
          newIframe(cross_site_origin),  // thirdPartyCrossSite
          newIframeCredentialless(same_site_origin),  // anonymousFrameSameSite
          null,  // anonymousFrameSameSite_AncestorBit
          newIframeCredentialless(
              cross_site_origin),  // anonymousFrameCrossSite
        ];
        // Set up nested SameSite frames for ancestor bit contexts.
        const setUpQueue = token();
        send(newIframe(cross_site_origin),
          child_frame_js(same_site_origin, "newIframe", setUpQueue));
        handles[FRAME_CONTEXT.thirdPartySameSite_AncestorBit] =
          await receive(setUpQueue);
        send(
            newIframeCredentialless(cross_site_origin),
            child_frame_js(
                same_site_origin, 'newIframeCredentialless', setUpQueue));
        handles[FRAME_CONTEXT.anonymousFrameSameSite_AncestorBit] =
          await receive(setUpQueue);

        const sendTo = (contexts, js_generator) => {
          // Send to all contexts in parallel to minimize timeout concerns.
          return Promise.all(contexts.map(async (context) => {
            const queue = token();
            const js_string = js_generator(queue, context);
            switch (context) {
              case FRAME_CONTEXT.firstParty:
                // Code is executed directly in this frame via eval() rather
                // than in a new context to avoid differences in API access.
                eval(`(async () => {${js_string}})()`);
                break;
              case FRAME_CONTEXT.thirdPartySameSite:
              case FRAME_CONTEXT.thirdPartyCrossSite:
              case FRAME_CONTEXT.anonymousFrameSameSite:
              case FRAME_CONTEXT.anonymousFrameCrossSite:
              case FRAME_CONTEXT.thirdPartySameSite_AncestorBit:
              case FRAME_CONTEXT.anonymousFrameSameSite_AncestorBit:
                send(handles[context], js_string);
                break;
              default:
                reject(`Cannot execute in context: ${context}`);
            }
            if (await receive(queue) != "done") {
              reject(`Script failed in frame ${context}: ${js_string}`);
            }
          }));
        };

        await func(t, sendTo);
      } catch (e) {
        reject(e);
      }
      resolve();
    });
  }, description);
}
