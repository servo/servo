// META: variant=?same-site
// META: variant=?cross-site
// META: timeout=long
// META: script=/common/get-host-info.sub.js
// META: script=/common/utils.js
// META: script=/common/dispatcher/dispatcher.js
// META: script=resources/about-blank-navigation.js

// A popup navigates its grandchild to about:blank and stores state in the
// grandchild's session history entry. The grandchild's parent has another
// origin. When the parent navigates the grandchild away and back, the
// about:blank document gets the popup's origin again, so the parent still
// can't read that state.

async function setUp(t) {
  const initiator = await openPopupContext(t, hosts.ORIGIN);
  const parent = await addIframeContext(initiator, variantOrigin);
  await addIframe(parent, `${variantOrigin}/common/blank.html`);
  const loaded = await expectIframeLoad(parent);
  await initiator.execute_script(() => {
    frames[0][0].location.href = "about:blank";
  });
  await loaded();
  return { initiator, parent };
}

async function navigateAwayAndBack(parent) {
  let loaded = await expectIframeLoad(parent);
  await parent.execute_script(url => {
    frames[0].location.href = url;
  }, [`${variantOrigin}/common/blank.html?next`]);
  await loaded();
  loaded = await expectIframeLoad(parent);
  await parent.execute_script(() => history.back());
  await loaded();
}

promise_test(async t => {
  const { initiator, parent } = await setUp(t);
  assert_equals(await initiator.execute_script(() => {
    try {
      frames[0][0].history.pushState({ secret: "initiator's" }, "");
    } catch (e) {
      return e.name;
    }
  }), undefined, "The initiator can add a session history entry with state");
  await navigateAwayAndBack(parent);
  assert_equals(await parent.execute_script(() => {
    try {
      return String(frames[0].history.state);
    } catch (e) {
      return e.name;
    }
  }), "SecurityError", "The parent can't read the state");
  assert_equals(await initiator.execute_script(() => {
    try {
      return frames[0][0].history.state?.secret;
    } catch (e) {
      return e.name;
    }
  }), "initiator's", "The initiator can read the state");
}, "history.state of an about:blank document stays with the initiator's origin after traversal");

promise_test(async t => {
  const { initiator, parent } = await setUp(t);
  assert_equals(await initiator.execute_script(() => {
    try {
      frames[0][0].navigation.updateCurrentEntry({ state: { secret: "initiator's" } });
    } catch (e) {
      return e.name;
    }
  }), undefined, "The initiator can store state in the session history entry");
  await navigateAwayAndBack(parent);
  assert_equals(await parent.execute_script(() => {
    try {
      return String(frames[0].navigation.currentEntry.getState());
    } catch (e) {
      return e.name;
    }
  }), "SecurityError", "The parent can't read the state");
  assert_equals(await initiator.execute_script(() => {
    try {
      return frames[0][0].navigation.currentEntry.getState()?.secret;
    } catch (e) {
      return e.name;
    }
  }), "initiator's", "The initiator can read the state");
}, "Navigation API state of an about:blank document stays with the initiator's origin after traversal");
