// NOTE about testing methodology:
// This test checks whether popovers are hidden *after* the appropriate de-hover
// delay. The delay used for testing is kept low, to avoid this test taking too
// long, but that means that sometimes on a slow bot/client, the delay can
// elapse before we are able to check the popover status. And that can make this
// test flaky. To avoid that, the msSinceMouseOver() function is used to check
// that not-too-much time has passed, and if it has, the test is simply skipped.

const hoverDelays = 100; // This needs to match the style block below.
const hoverWaitTime = 200; // How long to wait to cover the delay for sure.

async function initialPopoverShow(invoker) {
  const popover = invoker.popoverTargetElement;
  assert_false(popover.matches(':popover-open'));
  await mouseOver(invoker); // Always start with the mouse over the invoker
  popover.showPopover();
  assert_true(popover.matches(':popover-open'));
}

function runHoverHideTest(popoverType, invokerType, invokerAction) {
  const descr = `popover=${popoverType}, invoker=${invokerType}, popovertargetaction=${invokerAction}`;
  promise_test(async (t) => {
    const {popover,invoker} = makeTestParts(t, popoverType, invokerType, invokerAction);
    await initialPopoverShow(invoker);
    await mouseOver(unrelated);
    let showing = popover.matches(':popover-open');
    if (msSinceMouseOver() >= hoverDelays)
      return; // The WPT runner was too slow.
    assert_true(showing,'popover shouldn\'t immediately hide');
    await mouseHover(unrelated,hoverWaitTime);
    assert_false(popover.matches(':popover-open'),'popover should hide after delay');
  },`The popover-hide-delay causes a popover to be hidden after a delay, ${descr}`);

  promise_test(async (t) => {
    const {popover,invoker} = makeTestParts(t, popoverType, invokerType, invokerAction);
    await initialPopoverShow(invoker);
    await mouseHover(popover,hoverWaitTime);
    assert_true(popover.matches(':popover-open'),'hovering the popover should keep it showing');
    await mouseOver(unrelated);
    let showing = popover.matches(':popover-open');
    if (msSinceMouseOver() >= hoverDelays)
      return; // The WPT runner was too slow.
    assert_true(showing,'subsequently hovering unrelated element shouldn\'t immediately hide the popover');
    await mouseHover(unrelated,hoverWaitTime);
    assert_false(popover.matches(':popover-open'),'hovering unrelated element should hide popover after delay');
  },`hovering the popover keeps it from being hidden, ${descr}`);

  promise_test(async (t) => {
    const {popover,invoker,mouseOverInvoker} = makeTestParts(t, popoverType, invokerType, invokerAction);
    await initialPopoverShow(invoker);
    assert_true(popover.matches(':popover-open'));
    await mouseHover(popover,hoverWaitTime);
    await mouseHover(mouseOverInvoker,hoverWaitTime);
    assert_true(popover.matches(':popover-open'),'Moving hover between invoker and popover should keep popover from being hidden');
    await mouseHover(unrelated,hoverWaitTime);
    assert_false(popover.matches(':popover-open'),'Moving hover to unrelated should finally hide the popover');
  },`hovering an invoking element keeps the popover from being hidden, ${descr}`);
}

function runHoverHideTestsForInvokerAction(invokerAction) {
  promise_test(async (t) => {
    const {popover,invoker} = makeTestParts(t, 'auto', 'button', 'show');
    assert_false(popover.matches(':popover-open'));
    assert_true(invoker.matches('[popovertarget]'),'invoker needs to match [popovertarget]');
    assert_equals(invoker.popoverTargetElement,popover,'invoker should point to popover');
    await mouseHover(invoker,hoverWaitTime);
    assert_true(msSinceMouseOver() >= hoverWaitTime,'waitForHoverTime should wait the specified time');
    assert_true(hoverWaitTime > hoverDelays,'hoverDelays is the value from CSS, hoverWaitTime should be longer than that');
    assert_equals(getComputedStyleTimeMs(invoker,'popoverShowDelay'),hoverDelays,'popover-show-delay is incorrect');
    assert_equals(getComputedStyleTimeMs(popover,'popoverHideDelay'),hoverDelays,'popover-hide-delay is incorrect');
  },'Test the harness');

  // Run for all invoker and popover types.
  ["button","input"].forEach(invokerType => {
    ["auto","hint","manual"].forEach(popoverType => {
      runHoverHideTest(popoverType, invokerType, invokerAction);
    });
  });
}

// Setup stuff
const unrelated = document.createElement('div');
unrelated.id = 'unrelated';
unrelated.textContent = 'Unrelated element';
const style = document.createElement('style');
document.body.append(unrelated,style);
style.textContent = `
  div, button, input {
    /* Fixed position everything to ensure nothing overlaps */
    position: fixed;
    max-height: 100px;
  }
  #unrelated {top: 100px;}
  [popovertarget] {
    top:200px;
    popover-show-delay: 100ms;
  }
  [popover] {
    width: 200px;
    height: 100px;
    top:300px;
    popover-hide-delay: 100ms;
  }
`;

function makeTestParts(t,popoverType,invokerType,invokerAction) {
  const popover = document.createElement('div');
  popover.id = `popover-${popoverType}-${invokerType}-${invokerAction}`;
  document.body.appendChild(popover);
  popover.popover = popoverType;
  assert_equals(popover.popover, popoverType, `Type ${popoverType} not supported`);
  const invoker = document.createElement(invokerType);
  document.body.appendChild(invoker);
  invoker.popoverTargetElement = popover;
  invoker.popoverTargetAction = invokerAction;
  assert_equals(invoker.popoverTargetAction, invokerAction, `Invoker action ${invokerAction} not supported`);
  let mouseOverInvoker;
  switch (invokerType) {
    case 'button':
      invoker.innerHTML = '<span><span data-note=nested_element>Click me</span></span>';
      mouseOverInvoker = invoker.firstElementChild.firstElementChild;
      assert_true(!!mouseOverInvoker);
      break;
    case 'input':
      invoker.type = 'button';
      mouseOverInvoker = invoker;
      break;
    default:
      assert_unreached('Invalid invokerType ' + invokerType);
      break;
  }
  t.add_cleanup(() => {popover.remove(); invoker.remove();});
  return {popover, invoker, mouseOverInvoker};
}

function getComputedStyleTimeMs(element,property) {
  // Times are in seconds, so just strip off the 's'.
  return Number(getComputedStyle(element)[property].slice(0,-1))*1000;
}
