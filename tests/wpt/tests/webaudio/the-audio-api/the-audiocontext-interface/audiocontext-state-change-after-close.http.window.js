'use strict';

promise_test(async t => {
  let audioContext = new AudioContext();
  await new Promise((resolve) => (audioContext.onstatechange = resolve));
  await audioContext.close();
  return promise_rejects_dom(
      t, 'InvalidStateError', audioContext.close(),
      'A closed AudioContext should reject calls to close');
}, 'Call close on a closed AudioContext');

promise_test(async t => {
  let audioContext = new AudioContext();
  await new Promise((resolve) => (audioContext.onstatechange = resolve));
  await audioContext.close();
  return promise_rejects_dom(
      t, 'InvalidStateError', audioContext.resume(),
      'A closed AudioContext should reject calls to resume');
}, 'Call resume on a closed AudioContext');

promise_test(async t => {
  let audioContext = new AudioContext();
  await new Promise((resolve) => (audioContext.onstatechange = resolve));
  await audioContext.close();
  return promise_rejects_dom(
      t, 'InvalidStateError', audioContext.suspend(),
      'A closed AudioContext should reject calls to suspend');
}, 'Call suspend on a closed AudioContext');
