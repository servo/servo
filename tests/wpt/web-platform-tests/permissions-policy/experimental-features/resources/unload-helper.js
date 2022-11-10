// Code used by controlling frame of the unload policy tests.

const MAIN_FRAME = 'main';
const SUBFRAME = 'sub';

async function isUnloadAllowed(remoteContextWrapper) {
  return remoteContextWrapper.executeScript(() => {
    return document.featurePolicy.allowsFeature('unload');
  });
}

// Checks whether a frame runs unload handlers.
// This checks the policy directly and also installs an unload handler and
// navigates the frame checking that the handler ran.
async function assertWindowRunsUnload(
    remoteContextWrapper, name, {shouldRunUnload}) {
  const maybeNot = shouldRunUnload ? '' : 'not ';
  assert_equals(
      await isUnloadAllowed(remoteContextWrapper), shouldRunUnload,
      `${name}: unload in ${name} should ${maybeNot}be allowed`);

  // Set up recording of whether unload handler ran.
  await remoteContextWrapper.executeScript((name) => {
    localStorage.setItem(name, 'did not run');
    addEventListener('unload', () => localStorage.setItem(name, 'did run'));
  }, [name]);

  // Navigate away and then back.
  const second = await remoteContextWrapper.navigateToNew();
  // Navigating back ensures that the unload has completed.
  // Also if the navigation is cross-site then we have to return
  // to the original origin in order to read the recorded unload.
  second.historyBack();

  // Check that unload handlers ran as expected.
  const recordedUnload = await remoteContextWrapper.executeScript(
      (name) => localStorage.getItem(name), [name]);
  assert_equals(
      recordedUnload, `did ${maybeNot}run`,
      `${name}: unload should ${maybeNot}have run`);
}
