'use strict';

// Runs a drag in a popup copy of the current test so the controller survives
// reload. Each HTML file supplies the element with id="source", its handlers,
// and its assertions. getExpectedData reads the original source; verifyDrop
// checks the replacement element after the new document receives the payload.
function testDragAcrossReload({dataType, getExpectedData, verifyDrop}) {
  if (new URLSearchParams(location.search).has('popup')) {
    // Only the opener owns a harness test.
    setup({explicit_done: true, explicit_timeout: true});
    window.addEventListener('error', event => {
      opener.postMessage({type: 'error', message: event.message},
                         location.origin);
    });
    window.addEventListener('load', () => {
      opener.postMessage({
        type: 'loaded',
        navigationType: performance.getEntriesByType('navigation')[0].type,
      },
                         location.origin);
    });
    document.getElementById('source').addEventListener('drop', event => {
      // Observe the fixture's drop handler, reading data during event dispatch.
      opener.postMessage({
        type: 'drop',
        data: event.dataTransfer.getData(dataType),
        navigationType: performance.getEntriesByType('navigation')[0].type,
      },
                         location.origin);
    });
    return;
  }

  promise_test(async t => {
    let popup;
    function waitForMessage(type) {
      return new Promise((resolve, reject) => {
        const onMessage = event => {
          if (event.source !== popup || event.origin !== location.origin) {
            return;
          }
          if (event.data.type === 'error') {
            window.removeEventListener('message', onMessage);
            reject(new Error(event.data.message));
          } else if (event.data.type === type) {
            window.removeEventListener('message', onMessage);
            resolve(event.data);
          }
        };
        window.addEventListener('message', onMessage);
        t.add_cleanup(() => window.removeEventListener('message', onMessage));
      });
    }

    const loaded = waitForMessage('loaded');
    await test_driver.bless('Open the drag source', () => {
      popup = window.open('?popup');
    });
    assert_not_equals(popup, null, 'The source window opens');
    t.add_cleanup(() => popup.close());
    await loaded;

    const originalDocument = popup.document;
    const source = originalDocument.getElementById('source');
    const expectedData = getExpectedData(source);
    const rect = source.getBoundingClientRect();
    const x = Math.floor(rect.x + rect.width / 2);
    const y = Math.floor(rect.y + rect.height / 2);
    const reloaded = waitForMessage('loaded');
    const dropped = waitForMessage('drop');
    let dropSettled = false;
    const markDropSettled = () => {
      dropSettled = true;
    };
    dropped.then(markDropSettled, markDropSettled);
    // WPT resets input between sequences, so keep the entire drag together.
    await new test_driver.Actions()
        .setContext(popup)
        .addPointer('reload-mouse', 'mouse')
        .pointerMove(x, y)
        .pointerDown()
        // macOS requires a hold before dragging a text selection.
        .pause(200, 'pointer')
        .pointerMove(x + 20, y + 20)
        // Heuristic reload wait; assertions require drop in the new document.
        // TODO: Synchronize on load when Actions can preserve input state.
        .pause(1000, 'pointer')
        .pointerMove(rect.right + 50, rect.bottom + 50)
        .pointerMove(x, y)
        // Deliver dragover before releasing the pointer.
        .pointerMove(x + 1, y + 1)
        .pointerUp()
        .send();
    const reloadMessage = await reloaded;
    assert_not_equals(popup.document, originalDocument,
                      'The document is replaced');
    assert_equals(reloadMessage.navigationType, 'reload',
                  'The navigation is a reload');
    await t.step_wait(
        () => dropSettled,
        'The reloaded document receives a drop after pointer release');
    const dropMessage = await dropped;
    assert_equals(dropMessage.navigationType, 'reload',
                  'The new document receives drop');
    const data = dataType === 'text/uri-list' ?
        dropMessage.data.replace(/\r\n$/, '') :
        dropMessage.data;
    assert_equals(data, expectedData, 'The drag data survives reload');
    await verifyDrop(popup.document.getElementById('source'), expectedData);
  }, document.title);
}
