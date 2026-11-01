'use strict';

// Registers a promise_test that checks removal during dragstart and verifies
// that the original payload reaches the drop target. The fixture must install
// its source-removal and drop handlers before the test runs.
function testDragSourceRemoval(source, target,
                               {dataType, expectedData, verifyDrop}) {
  promise_test(async t => {
    if (document.readyState !== 'complete') {
      await new EventWatcher(t, window, 'load').wait_for('load');
    }

    assert_true(source.isConnected, 'The source is initially in the document');
    let removedDuringDragStart = false;
    const onDragStart = () => {
      // The fixture's dragstart handler runs first and removes the source.
      removedDuringDragStart = !source.isConnected;
    };
    source.addEventListener('dragstart', onDragStart);
    t.add_cleanup(() => {
      source.removeEventListener('dragstart', onDragStart);
    });

    let droppedData;
    const dropped = new Promise(resolve => {
      const onDrop = event => {
        // The data store is only readable while the drop event is dispatched.
        droppedData = event.dataTransfer.getData(dataType);
        resolve();
      };
      target.addEventListener('drop', onDrop, {once: true});
      t.add_cleanup(() => {
        target.removeEventListener('drop', onDrop);
      });
    });

    await Promise.all([dropped, dragAndDropWithLayoutShift(source, target)]);

    assert_true(removedDuringDragStart, 'dragstart removed the source');
    assert_false(source.isConnected, 'The source remains removed after drop');
    if (dataType === 'text/uri-list') {
      droppedData = droppedData.replace(/\r\n$/, '');
    }
    assert_equals(droppedData, expectedData, 'The drag data survives removal');
    await verifyDrop();
  }, document.title);
}

// Drop handler that appends an image to the target element using the single
// URL in text/uri-list. Used separately from assertions so the test observes
// the actual drop handler's result.
function addDroppedImage(event) {
  const image = document.createElement('img');
  image.src = event.dataTransfer.getData('text/uri-list').replace(/\r\n$/, '');
  event.currentTarget.appendChild(image);
}

// Registers a source-removal test for an image payload. Checks the transferred
// URL and that the target's drop handler inserted an image that decodes.
function testImageSourceRemoval(source, target, expectedURL = source.src) {
  testDragSourceRemoval(source, target, {
    dataType: 'text/uri-list',
    expectedData: expectedURL,
    verifyDrop: async () => {
      const image = target.querySelector('img');
      assert_not_equals(image, null, 'The drop handler inserts an image');
      assert_equals(image.src, expectedURL,
                    'The dropped image has the source URL');
      await image.decode();
    },
  });
}
