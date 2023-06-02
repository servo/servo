/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for resource interface bindings`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

import { declareEntrypoint, kResourceEmitters, kResourceKindsA, kResourceKindsB } from './util.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('single_entry_point')
  .desc(
    `Test that two different resource variables in a shader must not have the same group and binding values, when considered as a pair.`
  )
  .params(u =>
    u
      .combine('stage', ['vertex', 'fragment', 'compute'])
      .combine('a_kind', kResourceKindsA)
      .combine('b_kind', kResourceKindsB)
      .combine('a_group', [0, 3])
      .combine('b_group', [0, 3])
      .combine('a_binding', [0, 3])
      .combine('b_binding', [0, 3])
      .combine('usage', ['direct', 'transitive'])
      .beginSubcases()
  )
  .fn(t => {
    const resourceA = kResourceEmitters.get(t.params.a_kind);
    const resourceB = kResourceEmitters.get(t.params.b_kind);
    const resources = `
${resourceA('resource_a', t.params.a_group, t.params.a_binding)}
${resourceB('resource_b', t.params.b_group, t.params.b_binding)}
`;
    const expect =
      t.params.a_group !== t.params.b_group || t.params.a_binding !== t.params.b_binding;

    if (t.params.usage === 'direct') {
      const code = `
${resources}
${declareEntrypoint('main', t.params.stage, '_ = resource_a; _ = resource_b;')}
`;
      t.expectCompileResult(expect, code);
    } else {
      const code = `
${resources}
fn use_a() { _ = resource_a; }
fn use_b() { _ = resource_b; }
${declareEntrypoint('main', t.params.stage, 'use_a(); use_b();')}
`;
      t.expectCompileResult(expect, code);
    }
  });

g.test('different_entry_points')
  .desc(
    `Test that resources may use the same binding points if exclusively accessed by different entry points.`
  )
  .params(u =>
    u
      .combine('a_stage', ['vertex', 'fragment', 'compute'])
      .combine('b_stage', ['vertex', 'fragment', 'compute'])
      .combine('a_kind', kResourceKindsA)
      .combine('b_kind', kResourceKindsB)
      .combine('usage', ['direct', 'transitive'])
      .beginSubcases()
  )
  .fn(t => {
    const resourceA = kResourceEmitters.get(t.params.a_kind);
    const resourceB = kResourceEmitters.get(t.params.b_kind);
    const resources = `
${resourceA('resource_a', /* group */ 0, /* binding */ 0)}
${resourceB('resource_b', /* group */ 0, /* binding */ 0)}
`;
    const expect = true; // Binding reuse between different entry points is fine.

    if (t.params.usage === 'direct') {
      const code = `
${resources}
${declareEntrypoint('main_a', t.params.a_stage, '_ = resource_a;')}
${declareEntrypoint('main_b', t.params.b_stage, '_ = resource_b;')}
`;
      t.expectCompileResult(expect, code);
    } else {
      const code = `
${resources}
fn use_a() { _ = resource_a; }
fn use_b() { _ = resource_b; }
${declareEntrypoint('main_a', t.params.a_stage, 'use_a();')}
${declareEntrypoint('main_b', t.params.b_stage, 'use_b();')}
`;
      t.expectCompileResult(expect, code);
    }
  });

g.test('binding_attributes')
  .desc(`Test that both @group and @binding attributes must both be declared.`)
  .params(u =>
    u
      .combine('stage', ['vertex', 'fragment', 'compute'])
      .combine('has_group', [true, false])
      .combine('has_binding', [true, false])
      .beginSubcases()
  )
  .fn(t => {
    const emitter = kResourceEmitters.get('uniform');
    const code = emitter(
      'R',
      t.params.has_group ? 0 : undefined,
      t.params.has_binding ? 0 : undefined
    );

    const expect = t.params.has_group && t.params.has_binding;
    t.expectCompileResult(expect, code);
  });
