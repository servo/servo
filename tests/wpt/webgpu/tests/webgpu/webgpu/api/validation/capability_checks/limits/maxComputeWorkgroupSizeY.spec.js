/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/import { kMaximumLimitBaseParams, makeLimitTestGroup } from './limit_utils.js';const limit = 'maxComputeWorkgroupSizeY';
export const { g, description } = makeLimitTestGroup(limit);

g.test('createComputePipeline,at_over').
desc(`Test using createComputePipeline(Async) at and over ${limit} limit`).
params(kMaximumLimitBaseParams.combine('async', [false, true])).
fn(async (t) => {
  const { limitTest, testValueName, async } = t.params;
  await t.testMaxComputeWorkgroupSize(limitTest, testValueName, async, 'Y');
});

g.test('validate,maxComputeInvocationsPerWorkgroup').
desc(`Test that ${limit} <= maxComputeInvocationsPerWorkgroup`).
fn((t) => {
  const { adapter, defaultLimit, adapterLimit } = t;
  t.expect(defaultLimit <= t.getDefaultLimit('maxComputeInvocationsPerWorkgroup'));
  t.expect(adapterLimit <= adapter.limits.maxComputeInvocationsPerWorkgroup);
});