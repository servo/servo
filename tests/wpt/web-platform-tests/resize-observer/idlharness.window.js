// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: script=resources/resizeTestHelper.js

'use strict';

// https://wicg.github.io/ResizeObserver/

idl_test(
  ['ResizeObserver'],
  ['dom', 'geometry'],
  async idl_array => {
    idl_array.add_objects({
      ResizeObserver: ['observer'],
      ResizeObserverEntry: ['entry'],
    });

    const div = document.createElement('div');
    document.body.appendChild(div);
    let helper = new ResizeTestHelper(
      "ResizeObserverEntry creator",
      [
        {
          setup: observer => {
            self.observer = observer;
            observer.observe(div);
            div.style.width = "5px";
          },
          notify: entries => {
            self.entry = entries[0];
            assert_equals(entries[0].contentRect.width, 5, "target width");
          }
        }
      ]);
    await helper.start();
  }
);
