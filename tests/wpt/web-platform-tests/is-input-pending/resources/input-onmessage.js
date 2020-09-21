// Responds to onmessage events from other frames to check for pending input.
onmessage = async e => {
  if (e.data !== 'check-input') return;

  const discreteOptions = new IsInputPendingOptions({ includeContinuous: false });
  const continuousOptions = new IsInputPendingOptions({ includeContinuous: true });

  // Use a reasonable time to wait after dispatching events, since we want to be
  // able to test for cases where isInputPending returns false.
  const DISPATCH_WAIT_TIME_MS = 500;

  // Wait a reasonable amount of time for the event to be enqueued.
  const end = performance.now() + DISPATCH_WAIT_TIME_MS;
  let hasDiscrete;
  let hasContinuous;
  do {
    hasDiscrete = navigator.scheduling.isInputPending(discreteOptions);
    hasContinuous = navigator.scheduling.isInputPending(continuousOptions);
  } while (performance.now() < end && !(hasDiscrete && hasContinuous));

  e.source.postMessage({
    discrete: hasDiscrete,
    continuous: hasContinuous,
  }, '*');
}
