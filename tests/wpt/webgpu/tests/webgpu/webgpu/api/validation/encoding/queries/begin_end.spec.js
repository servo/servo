/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation for encoding begin/endable queries.
`;import { makeTestGroup } from '../../../../../common/framework/test_group.js';
import { ValidationTest } from '../../validation_test.js';

import { beginRenderPassWithQuerySet, createQuerySetWithType } from './common.js';

export const g = makeTestGroup(ValidationTest);

g.test('occlusion_query,begin_end_balance').
desc(
  `
Tests that begin/end occlusion queries mismatch on render pass:
- begin n queries, then end m queries, for various n and m.
  `
).
paramsSubcasesOnly([
{ begin: 0, end: 1 },
{ begin: 1, end: 0 },
{ begin: 1, end: 1 }, // control case
{ begin: 1, end: 2 },
{ begin: 2, end: 1 }]
).
fn((t) => {
  const { begin, end } = t.params;

  const occlusionQuerySet = createQuerySetWithType(t, 'occlusion', 2);

  const encoder = t.createEncoder('render pass', { occlusionQuerySet });
  for (let i = 0; i < begin; i++) {
    encoder.encoder.beginOcclusionQuery(i);
  }
  for (let j = 0; j < end; j++) {
    encoder.encoder.endOcclusionQuery();
  }
  encoder.validateFinishAndSubmit(begin === end, true);
});

g.test('occlusion_query,begin_end_invalid_nesting').
desc(
  `
Tests the invalid nesting of begin/end occlusion queries:
- begin index 0, end, begin index 0, end (control case)
- begin index 0, begin index 0, end, end
- begin index 0, begin index 1, end, end
  `
).
paramsSubcasesOnly([
{ calls: [0, 'end', 1, 'end'], _valid: true }, // control case
{ calls: [0, 0, 'end', 'end'], _valid: false },
{ calls: [0, 1, 'end', 'end'], _valid: false }]
).
fn((t) => {
  const { calls, _valid } = t.params;

  const occlusionQuerySet = createQuerySetWithType(t, 'occlusion', 2);

  const encoder = t.createEncoder('render pass', { occlusionQuerySet });
  for (const i of calls) {
    if (i !== 'end') {
      encoder.encoder.beginOcclusionQuery(i);
    } else {
      encoder.encoder.endOcclusionQuery();
    }
  }
  encoder.validateFinishAndSubmit(_valid, true);
});

g.test('occlusion_query,disjoint_queries_with_same_query_index').
desc(
  `
Tests that two disjoint occlusion queries cannot be begun with same query index on same render pass:
- begin index 0, end, begin index 0, end
- call on {same (invalid), different (control case)} render pass
  `
).
paramsSubcasesOnly((u) => u.combine('isOnSameRenderPass', [false, true])).
fn((t) => {
  const querySet = createQuerySetWithType(t, 'occlusion', 1);

  const encoder = t.device.createCommandEncoder();
  const pass = beginRenderPassWithQuerySet(t, encoder, querySet);
  pass.beginOcclusionQuery(0);
  pass.endOcclusionQuery();

  if (t.params.isOnSameRenderPass) {
    pass.beginOcclusionQuery(0);
    pass.endOcclusionQuery();
    pass.end();
  } else {
    pass.end();
    const otherPass = beginRenderPassWithQuerySet(t, encoder, querySet);
    otherPass.beginOcclusionQuery(0);
    otherPass.endOcclusionQuery();
    otherPass.end();
  }

  t.expectValidationError(() => {
    encoder.finish();
  }, t.params.isOnSameRenderPass);
});

g.test('nesting').
desc(
  `
Tests that whether it's allowed to nest various types of queries:
- call {occlusion, timestamp} query in same type or other type.
  `
).
paramsSubcasesOnly([
{ begin: 'occlusion', nest: 'timestamp', end: 'occlusion', _valid: true },
{ begin: 'occlusion', nest: 'occlusion', end: 'occlusion', _valid: false },
{ begin: 'timestamp', nest: 'occlusion', end: 'occlusion', _valid: true }]
).
unimplemented();