function createTopLayerElement(t,topLayerType) {
  let element, show, showing;
  switch (topLayerType) {
    case 'dialog':
      element = document.createElement('dialog');
      show = () => element.showModal();
      showing = () => element.matches(':modal');
      break;
    case 'fullscreen':
      element = document.createElement('div');
      show = async (topmostElement) => {
        // Be sure to add user activation to the topmost visible target:
        await blessTopLayer(topmostElement);
        await element.requestFullscreen();
      };
      showing = () => document.fullscreenElement === element;
      break;
    default:
      assert_unreached('Invalid top layer type');
  }
  t.add_cleanup(() => element.remove());
  return {element,show,showing};
}
function runTopLayerTests(testCases, testAnchorAttribute) {
  testAnchorAttribute = testAnchorAttribute || false;
  testCases.forEach(test => {
    const description = test.firstChild.data.trim();
    assert_equals(test.querySelectorAll('.target').length,1,'There should be exactly one target');
    const target = test.querySelector('.target');
    assert_true(!!target,'Invalid test case');
    const popovers = Array.from(test.querySelectorAll('[popover]'));
    assert_true(popovers.length > 0,'No popovers found');
    ['dialog','fullscreen'].forEach(topLayerType => {
      promise_test(async t => {
        const {element,show,showing} = createTopLayerElement(t,topLayerType);
        target.appendChild(element);

        // Show the popovers.
        t.add_cleanup(() => popovers.forEach(popover => popover.hidePopover()));
        popovers.forEach(popover => popover.showPopover());
        popovers.forEach(popover => assert_true(popover.matches(':popover-open'),'All popovers should be open'));

        // Activate the top layer element.
        await show(popovers[popovers.length-1]);
        assert_true(showing());
        popovers.forEach(popover => assert_equals(popover.matches(':popover-open'),popover.dataset.stayOpen==='true','Incorrect behavior'));

        // Add another popover within the top layer element and make sure entire stack stays open.
        const newPopover = document.createElement('div');
        t.add_cleanup(() => newPopover.remove());
        newPopover.popover = popoverHintSupported() ? 'hint' : 'auto';
        element.appendChild(newPopover);
        popovers.forEach(popover => assert_equals(popover.matches(':popover-open'),popover.dataset.stayOpen==='true','Adding another popover shouldn\'t change anything'));
        assert_true(showing(),'top layer element should still be top layer');
        newPopover.showPopover();
        assert_true(newPopover.matches(':popover-open'));
        popovers.forEach(popover => assert_equals(popover.matches(':popover-open'),popover.dataset.stayOpen==='true','Showing the popover shouldn\'t change anything'));
        assert_true(showing(),'top layer element should still be top layer');
      },`${description} with ${topLayerType}`);

      promise_test(async t => {
        const {element,show,showing} = createTopLayerElement(t,topLayerType);
        element.popover = popoverHintSupported() ? 'hint' : 'auto';
        target.appendChild(element);

        // Show the popovers.
        t.add_cleanup(() => popovers.forEach(popover => popover.hidePopover()));
        popovers.forEach(popover => popover.showPopover());
        popovers.forEach(popover => assert_true(popover.matches(':popover-open'),'All popovers should be open'));
        const targetWasOpenPopover = target.matches(':popover-open');

        // Show the top layer element as a popover first.
        element.showPopover();
        assert_true(element.matches(':popover-open'),'element should be open as a popover');
        assert_equals(target.matches(':popover-open'),targetWasOpenPopover,'target shouldn\'t change popover state');

        try {
          await show(element);
          assert_unreached('It is an error to activate a top layer element that is already a showing popover');
        } catch (e) {
          // We expect an InvalidStateError for dialogs, and a TypeError for fullscreens.
          // Anything else should fall through to the test harness.
          if (e.name !== 'InvalidStateError' && e.name !== 'TypeError') {
            throw e;
          }
        }
      },`${description} with ${topLayerType}, top layer element *is* a popover`);

      if (testAnchorAttribute) {
        promise_test(async t => {
          const {element,show,showing} = createTopLayerElement(t,topLayerType);
          element.anchorElement = target;
          document.body.appendChild(element);

          // Show the popovers.
          t.add_cleanup(() => popovers.forEach(popover => popover.hidePopover()));
          popovers.forEach(popover => popover.showPopover());
          popovers.forEach(popover => assert_true(popover.matches(':popover-open'),'All popovers should be open'));

          // Activate the top layer element.
          await show(popovers[popovers.length-1]);
          assert_true(showing());
          popovers.forEach(popover => assert_equals(popover.matches(':popover-open'),popover.dataset.stayOpen==='true','Incorrect behavior'));
        },`${description} with ${topLayerType}, anchor attribute`);
      }
    });
  });
}
