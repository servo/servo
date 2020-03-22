// Number milliseconds to wait for CSS resources to load.
const numMillisecondsWait = 50;

// We use requestAnimationFrame() calls to force the user agent to paint and give enough
// time for FCP to show up in the performance timeline. Hence, set |numFramesWaiting| to
// 3 and use that constant whenever the test needs to wait for the next paint to occur.
const numFramesWaiting = 3;

function waitTime(t) {
  return new Promise(resolve => t.step_timeout(resolve, numMillisecondsWait));
}

function waitForAnimationFrames(count) {
  return new Promise(resolve => {
    if (count-- <= 0) {
      resolve();
    } else {
      requestAnimationFrame(() => {
        waitForAnimationFrames(count).then(resolve);
      });
    }
  });
}

// Asserts that there is currently no FCP reported, even after some wait.
async function assertNoFirstContentfulPaint(t) {
  if (t)
    await waitTime(t);

  await waitForAnimationFrames(numFramesWaiting);
  assert_equals(performance.getEntriesByName('first-contentful-paint').length, 0, 'First contentful paint marked too early. ');
}

// Asserts that FCP is reported, possibly after some wait. The wait is needed
// because sometimes the FCP relies on some CSS resources to finish loading.
async function assertFirstContentfulPaint(t) {
  if (t)
    await waitTime(t);
  await waitForAnimationFrames(numFramesWaiting);
  assert_equals(performance.getEntriesByName('first-contentful-paint').length, 1, 'Expected first contentful paint not found. ');
}

async function test_fcp(label) {
  const style = document.createElement('style');
  document.head.appendChild(style);
  await promise_test(async t => {
    assert_precondition(window.PerformancePaintTiming, "Paint Timing isn't supported.");
    const main = document.getElementById('main');
    await new Promise(r => window.addEventListener('load', r));
    await assertNoFirstContentfulPaint();
    main.className = 'preFCP';
    await assertNoFirstContentfulPaint();
    main.className = 'contentful';
    await assertFirstContentfulPaint();
  }, label);
}
