/**
 * AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
 **/ export const description = `Validation tests for diagnostic directive and attribute`;
import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { keysOf } from '../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

const kSpecDiagnosticRules = ['derivative_uniformity'];
const kSpecDiagnosticSeverities = ['off', 'info', 'warning', 'error'];
const kDiagnosticTypes = ['attribute', 'directive'];

const kBadSeverities = ['none', 'warn', 'goose', 'fatal', 'severe'];
const kBadSingleTokenRules = ['unknown', 'blahblahblah', 'derivative_uniform'];

function generateDiagnostic(type, severity, rule) {
  const diagnostic = `diagnostic(${severity}, ${rule})`;
  if (type === 'directive') {
    return diagnostic;
  } else {
    return '@' + diagnostic;
  }
}

const kValidLocations = {
  module: diag => `${diag};`,
  function: diag => `${diag} fn foo() { }`,
  compound: diag => `fn foo() { ${diag} { } }`,
  if_stmt: diag => `fn foo() { ${diag} if true { } }`,
  if_then: diag => `fn foo() { if true ${diag} { } }`,
  if_else: diag => `fn foo() { if true { } else ${diag} { } }`,
  switch_stmt: diag => `fn foo() { ${diag} switch 0 { default { } } }`,
  switch_body: diag => `fn foo() { switch 0 ${diag} { default { } } }`,
  switch_default: diag => `fn foo() { switch 0 { default ${diag} { } } }`,
  switch_case: diag => `fn foo() { switch 0 { case 0 ${diag} { } default { } } }`,
  loop_stmt: diag => `fn foo() { ${diag} loop { break; } }`,
  loop_body: diag => `fn foo() { loop ${diag} { break; } }`,
  loop_continuing: diag => `fn foo() { loop { continuing ${diag} { break if true; } } }`,
  while_stmt: diag => `fn foo() { ${diag} while true { break; } }`,
  while_body: diag => `fn foo() { while true ${diag} { break; } }`,
  for_stmt: diag => `fn foo() { ${diag} for (var i = 0; i < 10; i++) { } }`,
  for_body: diag => `fn foo() { for (var i = 0; i < 10; i++) ${diag} { } }`,
};

const kInvalidLocations = {
  module_var: diag => `${diag} var<private> x : u32;`,
  module_const: diag => `${diag} const x = 0;`,
  module_override: diag => `${diag} override x : u32;`,
  struct: diag => `${diag} struct S { x : u32 }`,
  struct_member: diag => ` struct S { ${diag} x : u32 }`,
  function_params: diag => `fn foo${diag}() { }`,
  function_var: diag => `fn foo() { ${diag} var x = 0; }`,
  function_let: diag => `fn foo() { ${diag} let x = 0; }`,
  function_const: diag => `fn foo() { ${diag} const x = 0; }`,
  pre_else: diag => `fn foo() { if true { } ${diag} else { } }`,
  pre_default: diag => `fn foo() { switch 0 { ${diag} default { } } }`,
  pre_case: diag => `fn foo() { switch 0 { ${diag} case 0 { } default { } } }`,
  pre_continuing: diag => `fn foo() { loop { ${diag} continuing { break if true; } } }`,
  pre_for_params: diag => `fn foo() { for ${diag} (var i = 0; i < 10; i++) { } }`,
};

const kNestedLocations = {
  compound: (d1, d2) => `${d1} fn foo() { ${d2} { } }`,
  if_stmt: (d1, d2) => `fn foo() { ${d1} if true ${d2} { } }`,
  switch_stmt: (d1, d2) => `fn foo() { ${d1} switch 0 ${d2} { default { } } }`,
  switch_body: (d1, d2) => `fn foo() { switch 0 ${d1} { default ${d2} { } } }`,
  switch_case: (d1, d2) => `fn foo() { switch 0 { case 0 ${d1} { } default ${d2} { } } }`,
  loop_stmt: (d1, d2) => `fn foo() { ${d1} loop ${d2} { break; } }`,
  while_stmt: (d1, d2) => `fn foo() { ${d1} while true ${d2} { break; } }`,
  for_stmt: (d1, d2) => `fn foo() { ${d1} for (var i = 0; i < 10; i++) ${d2} { } }`,
};

g.test('valid_params')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests required accepted diagnostic parameters`)
  .params(u =>
    u
      .combine('severity', kSpecDiagnosticSeverities)
      .combine('rule', kSpecDiagnosticRules)
      .combine('type', kDiagnosticTypes)
  )
  .fn(t => {
    const diag = generateDiagnostic(t.params.type, t.params.severity, t.params.rule);
    let code = ``;
    if (t.params.type === 'directive') {
      code = kValidLocations['module'](diag);
    } else {
      code = kValidLocations['function'](diag);
    }
    t.expectCompileResult(true, code);
  });

g.test('invalid_severity')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests invalid severities are rejected`)
  .params(u => u.combine('severity', kBadSeverities).combine('type', kDiagnosticTypes))
  .fn(t => {
    const diag = generateDiagnostic(t.params.type, t.params.severity, 'derivative_uniformity');
    let code = ``;
    if (t.params.type === 'directive') {
      code = kValidLocations['module'](diag);
    } else {
      code = kValidLocations['function'](diag);
    }
    t.expectCompileResult(false, code);
  });

g.test('warning_unknown_rule')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests unknown single token rules issue a warning`)
  .params(u => u.combine('type', kDiagnosticTypes).combine('rule', kBadSingleTokenRules))
  .fn(t => {
    const diag = generateDiagnostic(t.params.type, 'info', t.params.rule);
    let code = ``;
    if (t.params.type === 'directive') {
      code = kValidLocations['module'](diag);
    } else {
      code = kValidLocations['function'](diag);
    }
    t.expectCompileWarning(true, code);
  });

g.test('valid_locations')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests valid locations`)
  .params(u => u.combine('type', kDiagnosticTypes).combine('location', keysOf(kValidLocations)))
  .fn(t => {
    const diag = generateDiagnostic(t.params.type, 'info', 'derivative_uniformity');
    const code = kValidLocations[t.params.location](diag);
    let res = true;
    if (t.params.type === 'directive') {
      res = t.params.location === 'module';
    } else {
      res = t.params.location !== 'module';
    }
    if (res === false) {
      t.expectCompileResult(true, kValidLocations[t.params.location](''));
    }
    t.expectCompileResult(res, code);
  });

g.test('invalid_locations')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests invalid locations`)
  .params(u => u.combine('type', kDiagnosticTypes).combine('location', keysOf(kInvalidLocations)))
  .fn(t => {
    const diag = generateDiagnostic(t.params.type, 'info', 'derivative_uniformity');
    t.expectCompileResult(true, kInvalidLocations[t.params.location](''));
    t.expectCompileResult(false, kInvalidLocations[t.params.location](diag));
  });

g.test('conflicting_directive')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests conflicts between directives`)
  .params(u => u.combine('s1', kSpecDiagnosticSeverities).combine('s2', kSpecDiagnosticSeverities))
  .fn(t => {
    const d1 = generateDiagnostic('directive', t.params.s1, 'derivative_uniformity');
    const d2 = generateDiagnostic('directive', t.params.s2, 'derivative_uniformity');
    const code = `${kValidLocations['module'](d1)}\n${kValidLocations['module'](d2)}`;
    t.expectCompileResult(t.params.s1 === t.params.s2, code);
  });

g.test('conflicting_attribute_same_location')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests conflicts between attributes`)
  .params(u =>
    u
      .combine('loc', keysOf(kValidLocations))
      .combine('s1', kSpecDiagnosticSeverities)
      .combine('s2', kSpecDiagnosticSeverities)
      .filter(u => {
        return u.loc !== 'module';
      })
  )
  .fn(t => {
    const d1 = generateDiagnostic('attribute', t.params.s1, 'derivative_uniformity');
    const d2 = generateDiagnostic('attribute', t.params.s2, 'derivative_uniformity');
    const diag = d1 + ' ' + d2;
    const code = `${kValidLocations[t.params.loc](diag)}`;
    t.expectCompileResult(t.params.s1 === t.params.s2, code);
  });

g.test('conflicting_attribute_different_location')
  .specURL('https://gpuweb.github.io/gpuweb/wgsl/#diagnostics')
  .desc(`Tests conflicts between attributes`)
  .params(u =>
    u
      .combine('loc', keysOf(kNestedLocations))
      .combine('s1', kSpecDiagnosticSeverities)
      .combine('s2', kSpecDiagnosticSeverities)
      .filter(u => {
        return u.s1 !== u.s2;
      })
  )
  .fn(t => {
    const d1 = generateDiagnostic('attribute', t.params.s1, 'derivative_uniformity');
    const d2 = generateDiagnostic('attribute', t.params.s2, 'derivative_uniformity');
    const code = `${kNestedLocations[t.params.loc](d1, d2)}`;
    t.expectCompileResult(true, code);
  });
