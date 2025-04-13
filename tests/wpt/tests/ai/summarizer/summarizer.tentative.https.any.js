// META: title=Summarizer
// META: script=../resources/util.js
// META: timeout=long

'use strict';

promise_test(async () => {
  assert_true(!!Summarizer);
}, 'Summarizer must be defined.');

promise_test(async () => {
  const availability = await Summarizer.availability({
    type: "tl;dr",
    format: "plain-text",
    length: "medium",
  });
  assert_not_equals(availability, "unavailable");
}, 'Summarizer.availability() is available');

promise_test(async () => {
  const availability = await Summarizer.availability({
    type: "tl;dr",
    format: "plain-text",
    length: "medium",
    expectedInputLanguages: ["en-GB"],
    expectedContextLanguages: ["en"],
    outputLanguage: "en",
  });
  assert_not_equals(availability, "unavailable");
}, 'Summarizer.availability() is available for supported languages');

promise_test(async () => {
  const availability = await Summarizer.availability({
    type: "tl;dr",
    format: "plain-text",
    length: "medium",
    expectedInputLanguages: ["es"], // not supported
    expectedContextLanguages: ["en"],
    outputLanguage: "es", // not supported
  });
  assert_equals(availability, "unavailable");
}, 'Summarizer.availability() returns no for unsupported languages');

promise_test(async () => {
  await testMonitor(Summarizer.create);
}, 'Summarizer.create() notifies its monitor on downloadprogress');

promise_test(async () => {
  const summarizer = await Summarizer.create({});
  const result = await summarizer.summarize(kTestPrompt);
  assert_equals(typeof result, "string");
  assert_greater_than(result.length, 0);
}, 'Summarizer.summarize() returns non-empty result');

promise_test(async () => {
  const summarizer = await Summarizer.create({});
  const result = await summarizer.measureInputUsage(kTestPrompt);
  assert_greater_than(result, 0);
}, 'Summarizer.measureInputUsage() returns non-empty result');

promise_test(async () => {
  const sharedContext = 'This is a shared context string';
  const summarizer = await Summarizer.create({sharedContext: sharedContext});
  assert_equals(summarizer.sharedContext, sharedContext);
}, 'Summarizer.sharedContext');

promise_test(async () => {
  const summarizer = await Summarizer.create({type: 'headline'});
  assert_equals(summarizer.type, 'headline');
}, 'Summarizer.type');

promise_test(async () => {
  const summarizer = await Summarizer.create({format: 'markdown'});
  assert_equals(summarizer.format, 'markdown');
}, 'Summarizer.format');

promise_test(async () => {
  const summarizer = await Summarizer.create({length: 'medium'});
  assert_equals(summarizer.length, 'medium');
}, 'Summarizer.length');

promise_test(async () => {
  const summarizer = await Summarizer.create({
    expectedInputLanguages: ['en']
  });
  assert_array_equals(summarizer.expectedInputLanguages, ['en']);
}, 'Summarizer.expectedInputLanguages');

promise_test(async () => {
  const summarizer = await Summarizer.create({
    expectedContextLanguages: ['en']
  });
  assert_array_equals(summarizer.expectedContextLanguages, ['en']);
}, 'Summarizer.expectedContextLanguages');

promise_test(async () => {
  const summarizer = await Summarizer.create({
    outputLanguage: 'en'
  });
  assert_equals(summarizer.outputLanguage, 'en');
}, 'Summarizer.outputLanguage');

promise_test(async () => {
  const summarizer = await Summarizer.create({});
  assert_equals(summarizer.expectedInputLanguages, null);
  assert_equals(summarizer.expectedContextLanguages, null);
  assert_equals(summarizer.outputLanguage, null);
}, 'Summarizer optional attributes return null');
