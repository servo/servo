/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
Validation tests for atomic builtins.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { keysOf } from '../../../../../../common/util/data_tables.js';
import { ShaderValidationTest } from '../../../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);





const kAtomicOps = {
  add: (a) => {
    return `atomicAdd(${a},1)`;
  },
  sub: (a) => {
    return `atomicSub(${a},1)`;
  },
  max: (a) => {
    return `atomicMax(${a},1)`;
  },
  min: (a) => {
    return `atomicMin(${a},1)`;
  },
  and: (a) => {
    return `atomicAnd(${a},1)`;
  },
  or: (a) => {
    return `atomicOr(${a},1)`;
  },
  xor: (a) => {
    return `atomicXor(${a},1)`;
  },
  load: (a) => {
    return `atomicLoad(${a})`;
  },
  store: (a) => {
    return `atomicStore(${a},1)`;
  },
  exchange: (a) => {
    return `atomicExchange(${a},1)`;
  },
  compareexchangeweak: (a) => {
    return `atomicCompareExchangeWeak(${a},1,1)`;
  }
};

g.test('stage').
specURL('https://www.w3.org/TR/WGSL/#atomic-rmw').
desc(
  `
Atomic built-in functions must not be used in a vertex shader stage.
`
).
params((u) =>
u.
combine('stage', ['fragment', 'vertex', 'compute']) //
.combine('atomicOp', keysOf(kAtomicOps))
).
fn((t) => {
  const atomicOp = kAtomicOps[t.params.atomicOp](`&a`);
  let code = `
@group(0) @binding(0) var<storage, read_write> a: atomic<i32>;
`;

  switch (t.params.stage) {
    case 'compute':
      code += `
@compute @workgroup_size(1,1,1) fn main() {
  ${atomicOp};
}`;
      break;

    case 'fragment':
      code += `
@fragment fn main() -> @location(0) vec4<f32> {
  ${atomicOp};
  return vec4<f32>();
}`;
      break;

    case 'vertex':
      code += `
@vertex fn vmain() -> @builtin(position) vec4<f32> {
  ${atomicOp};
  return vec4<f32>();
}`;
      break;
  }

  const pass = t.params.stage !== 'vertex';
  t.expectCompileResult(pass, code);
});

function generateAtomicCode(
type,
access,
aspace,
style,
op)
{
  let moduleVar = ``;
  let functionVar = ``;
  let param = ``;
  let aParam = ``;
  if (style === 'var') {
    aParam = `&a`;
    switch (aspace) {
      case 'storage':
        moduleVar = `@group(0) @binding(0) var<storage, ${access}> a : atomic<${type}>;\n`;
        break;
      case 'workgroup':
        moduleVar = `var<workgroup> a : atomic<${type}>;\n`;
        break;
      case 'uniform':
        moduleVar = `@group(0) @binding(0) var<uniform> a : atomic<${type}>;\n`;
        break;
      case 'private':
        moduleVar = `var<private> a : atomic<${type}>;\n`;
        break;
      case 'function':
        functionVar = `var a : atomic<${type}>;\n`;
        break;
      default:
        break;
    }
  } else {
    const aspaceParam = aspace === 'storage' ? `, ${access}` : ``;
    param = `p : ptr<${aspace}, atomic<${type}>${aspaceParam}>`;
    aParam = `p`;
  }

  return `
${moduleVar}
fn foo(${param}) {
  ${functionVar}
  ${kAtomicOps[op](aParam)};
}
`;
}

g.test('atomic_parameterization').
desc('Tests the valid atomic parameters').
params((u) =>
u.
combine('op', keysOf(kAtomicOps)).
beginSubcases().
combine('aspace', ['storage', 'workgroup', 'private', 'uniform', 'function']).
combine('access', ['read', 'read_write']).
combine('type', ['i32', 'u32']).
combine('style', ['param', 'var']).
filter((t) => {
  switch (t.aspace) {
    case 'uniform':
      return t.style === 'param' && t.access === 'read';
    case 'workgroup':
      return t.access === 'read_write';
    case 'function':
    case 'private':
      return t.style === 'param' && t.access === 'read_write';
    default:
      return true;
  }
})
).
fn((t) => {
  if (
  t.params.style === 'param' &&
  !(t.params.aspace === 'function' || t.params.aspace === 'private'))
  {
    t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');
  }

  const aspaceOK = t.params.aspace === 'storage' || t.params.aspace === 'workgroup';
  const accessOK = t.params.access === 'read_write';
  t.expectCompileResult(
    aspaceOK && accessOK,
    generateAtomicCode(
      t.params.type,
      t.params.access,
      t.params.aspace,
      t.params.style,
      t.params.op
    )
  );
});

g.test('data_parameters').
desc('Validates that data parameters must match atomic type (or be implicitly convertible)').
params((u) =>
u.
combine('op', [
'atomicStore',
'atomicAdd',
'atomicSub',
'atomicMax',
'atomicMin',
'atomicAnd',
'atomicOr',
'atomicXor',
'atomicExchange',
'atomicCompareExchangeWeak1',
'atomicCompareExchangeWeak2']
).
beginSubcases().
combine('atomicType', ['i32', 'u32']).
combine('dataType', ['i32', 'u32', 'f32', 'AbstractInt'])
).
fn((t) => {
  let dataValue = '';
  switch (t.params.dataType) {
    case 'i32':
      dataValue = '1i';
      break;
    case 'u32':
      dataValue = '1u';
      break;
    case 'f32':
      dataValue = '1f';
      break;
    case 'AbstractInt':
      dataValue = '1';
      break;
  }
  let op = '';
  switch (t.params.op) {
    case 'atomicCompareExchangeWeak1':
      op = `atomicCompareExchangeWeak(&a, ${dataValue}, 1)`;
      break;
    case 'atomicCompareExchangeWeak2':
      op = `atomicCompareExchangeWeak(&a, 1, ${dataValue})`;
      break;
    default:
      op = `${t.params.op}(&a, ${dataValue})`;
      break;
  }
  const code = `
var<workgroup> a : atomic<${t.params.atomicType}>;
fn foo() {
  ${op};
}
`;

  const expect = t.params.atomicType === t.params.dataType || t.params.dataType === 'AbstractInt';
  t.expectCompileResult(expect, code);
});

g.test('return_types').
desc('Validates return types of atomics').
params((u) =>
u.
combine('op', keysOf(kAtomicOps)).
beginSubcases().
combine('atomicType', ['i32', 'u32']).
combine('returnType', ['i32', 'u32', 'f32'])
).
fn((t) => {
  let op = `${kAtomicOps[t.params.op]('&a')}`;
  switch (t.params.op) {
    case 'compareexchangeweak':
      op = `let tmp : ${t.params.returnType} = ${op}.old_value`;
      break;
    default:
      op = `let tmp : ${t.params.returnType} = ${op}`;
      break;
  }
  const code = `
var<workgroup> a : atomic<${t.params.atomicType}>;
fn foo() {
  ${op};
}
`;

  const expect = t.params.atomicType === t.params.returnType && t.params.op !== 'store';
  t.expectCompileResult(expect, code);
});

g.test('non_atomic').
desc('Test that non-atomic integers are rejected by all atomic functions.').
params((u) =>
u.
combine('op', keysOf(kAtomicOps)).
combine('addrspace', ['storage', 'workgroup']).
combine('type', ['i32', 'u32']).
combine('atomic', [true, false])
).
fn((t) => {
  let type = t.params.type;
  if (t.params.atomic) {
    type = `atomic<${type}>`;
  }

  let decl = '';
  if (t.params.addrspace === 'storage') {
    decl = `@group(0) @binding(0) var<storage, read_write> a : ${type}`;
  } else if (t.params.addrspace === 'workgroup') {
    decl = `var<workgroup> a : ${type}`;
  }

  const op = `${kAtomicOps[t.params.op]('&a')}`;
  const code = `
${decl};
fn foo() {
  ${op};
}
`;

  t.expectCompileResult(t.params.atomic, code);
});