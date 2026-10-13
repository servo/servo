// META: title=DecisionModel
// META: script=/resources/testdriver.js
// META: script=/resources/testdriver-vendor.js
// META: script=../resources/util.js

'use strict';

const kSampleSchema = {
  context: 'Enterprise customer support ticket router.',
  expectedInputs: [{type: 'text', languages: ['en']}],
  questions: [
    {
      id: 'is_urgent',
      type: 'boolean',
      prompt: 'Does this ticket require immediate incident response?',
    },
    {
      id: 'category',
      type: 'choice',
      prompt: 'Select the primary support department.',
      options: [
        {label: 'bug', description: 'Production crash or software defect'},
        {label: 'billing', description: 'Invoice or double-charge issue'},
        {label: 'feature_request', description: 'Enhancement request'},
      ],
    },
    {
      id: 'severity',
      type: 'score',
      prompt: 'Rate the business impact from 1 (minimal) to 5 (critical).',
    },
  ],
};

promise_test(async t => {
  const availability = await DecisionModel.availability(kSampleSchema);
  assert_in_array(availability,
                  ['available', 'downloadable', 'downloading', 'unavailable']);
}, 'DecisionModel.availability() returns a valid Availability enum value');

promise_test(async t => {
  const decisionModel = await createDecisionModel(kSampleSchema);
  t.add_cleanup(() => decisionModel.destroy());
  assert_equals(decisionModel.preference, 'auto');

  const result = await decisionModel.decide(
      'Production database connections are failing with timeout errors.');
  assert_equals(typeof result, 'object');

  const {is_urgent, category, severity} = result;
  assert_equals(is_urgent.id, 'is_urgent');
  assert_in_array(is_urgent.label, ['true', 'false']);
  assert_equals(typeof is_urgent.confidence, 'number');
  assert_equals(typeof is_urgent.probabilities, 'object');
  assert_false(Array.isArray(is_urgent.probabilities));
  assert_array_equals(Object.keys(is_urgent.probabilities), ['true', 'false']);
  for (const p of Object.values(is_urgent.probabilities)) {
    assert_equals(typeof p, 'number');
  }
  assert_approx_equals(
      Object.values(is_urgent.probabilities).reduce((sum, p) => sum + p, 0),
      1.0, 1e-4);

  assert_equals(category.id, 'category');
  assert_in_array(category.label, ['bug', 'billing', 'feature_request']);
  assert_equals(typeof category.confidence, 'number');
  assert_equals(typeof category.probabilities, 'object');
  assert_false(Array.isArray(category.probabilities));
  assert_array_equals(Object.keys(category.probabilities),
                      ['bug', 'billing', 'feature_request']);
  for (const p of Object.values(category.probabilities)) {
    assert_equals(typeof p, 'number');
  }

  assert_equals(severity.id, 'severity');
  assert_in_array(severity.label, ['1', '2', '3', '4', '5']);
  assert_equals(typeof severity.expectedScore, 'number');
  assert_equals(typeof severity.confidence, 'number');
  assert_equals(typeof severity.probabilities, 'object');
  assert_false(Array.isArray(severity.probabilities));
  assert_array_equals(Object.keys(severity.probabilities),
                      ['1', '2', '3', '4', '5']);
  for (const p of Object.values(severity.probabilities)) {
    assert_equals(typeof p, 'number');
  }
}, 'DecisionModel.create() and decisionModel.decide() return a record keyed by question id with probabilities keyed by option label');

promise_test(async t => {
  const customSchema = {
    questions: [
      {
        id: 'custom_route',
        type: 'choice',
        prompt: 'Route request to team.',
        options: [{label: 'infra'}, {label: 'security'}],
      },
      {
        id: 'custom_rating',
        type: 'score',
        prompt: 'Rate priority on a 10/20/30 scale.',
        options: [{label: '10'}, {label: '20'}, {label: '30'}],
      },
    ],
  };
  const model = await createDecisionModel(customSchema);
  t.add_cleanup(() => model.destroy());

  const result = await model.decide('Investigate anomalous outbound traffic.');
  assert_array_equals(Object.keys(result.custom_route.probabilities),
                      ['infra', 'security']);
  assert_array_equals(Object.keys(result.custom_rating.probabilities),
                      ['10', '20', '30']);
  assert_greater_than_equal(result.custom_rating.expectedScore, 10);
  assert_less_than_equal(result.custom_rating.expectedScore, 30);
}, 'DecisionModel dynamically reflects custom choice and score question schemas');

promise_test(async t => {
  const decisionModel = await createDecisionModel({
    questions: [
      ...kSampleSchema.questions,
      {
        id: 'sentiment',
        type: 'choice',
        prompt: 'Determine the overall customer sentiment.',
        options: [
          {label: 'positive'},
          {label: 'negative'},
        ],
      },
    ],
  });
  t.add_cleanup(() => decisionModel.destroy());

  const result = await decisionModel.decide(
      'Production database connections are failing with timeout errors.');
  assert_equals(Object.keys(result).length, 4);
  assert_equals(result.sentiment.id, 'sentiment');
  assert_in_array(result.sentiment.label, ['positive', 'negative']);
  assert_equals(Object.keys(result.sentiment.probabilities).length, 2);
}, 'decisionModel.decide() returns one decision per create-time question');

promise_test(async t => {
  await test_driver.bless();
  const invalidSchemas = [
    {questions: []},
    {questions: [{id: 'q1', type: 'rhetorical', prompt: 'Invalid type'}]},
    {questions: [{id: '   ', type: 'boolean', prompt: 'Blank id'}]},
    {questions: [{id: 'q1', type: 'boolean', prompt: '   '}]},
    {
      questions: [
        {id: 'dup', type: 'boolean', prompt: 'First question'},
        {id: 'dup', type: 'boolean', prompt: 'Duplicate id'},
      ],
    },
    {
      questions: [{
        id: 'bool_with_options',
        type: 'boolean',
        prompt: 'Boolean questions must not specify options',
        options: [{label: 'yes'}, {label: 'no'}],
      }],
    },
    {
      questions: [{
        id: 'choice_without_options',
        type: 'choice',
        prompt: 'Choice without options',
      }],
    },
    {
      questions: [{
        id: 'choice_with_single_option',
        type: 'choice',
        prompt: 'Choice with one option',
        options: [{label: 'only_one'}],
      }],
    },
    {
      questions: [{
        id: 'choice_with_duplicate_options',
        type: 'choice',
        prompt: 'Choice with duplicate options',
        options: [{label: 'dup'}, {label: 'dup'}],
      }],
    },
    {
      questions: [{
        id: 'score_with_single_option',
        type: 'score',
        prompt: 'Score with one option',
        options: [{label: '1'}],
      }],
    },
  ];

  for (const schema of invalidSchemas) {
    await promise_rejects_js(t, TypeError, DecisionModel.availability(schema));
    await promise_rejects_js(t, TypeError, DecisionModel.create(schema));
  }
  await promise_rejects_js(t, TypeError, DecisionModel.create({}));

  const invalidLanguageSchema = {
    expectedInputs: [{type: 'text', languages: ['invalid_bcp47_tag!']}],
    questions: [{id: 'q1', type: 'boolean', prompt: 'Valid type'}],
  };
  await promise_rejects_js(t, RangeError,
                           DecisionModel.availability(invalidLanguageSchema));
  await promise_rejects_js(t, RangeError,
                           DecisionModel.create(invalidLanguageSchema));
}, 'DecisionModel rejects malformed question schemas with TypeError and invalid BCP-47 language tags with RangeError');

promise_test(async t => {
  const decisionModel = await createDecisionModel(kSampleSchema);
  t.add_cleanup(() => decisionModel.destroy());

  await promise_rejects_js(t, TypeError, decisionModel.decide(''));
  await promise_rejects_js(t, TypeError, decisionModel.decide('   '));
}, 'decisionModel.decide() rejects empty or whitespace-only input with TypeError');

promise_test(async t => {
  const decisionModel = await createDecisionModel(kSampleSchema);
  t.add_cleanup(() => decisionModel.destroy());

  const controller = new AbortController();
  controller.abort();
  await promise_rejects_dom(
      t, 'AbortError',
      decisionModel.decide('Aborted input', {signal: controller.signal}));
}, 'decisionModel.decide() rejects with AbortError when passed an aborted AbortSignal');

promise_test(async t => {
  const decisionModel = await createDecisionModel(kSampleSchema);
  decisionModel.destroy();

  await promise_rejects_dom(
      t, 'InvalidStateError',
      decisionModel.decide('Input after session destruction'));
}, 'decisionModel.decide() rejects with InvalidStateError after destroy()');
