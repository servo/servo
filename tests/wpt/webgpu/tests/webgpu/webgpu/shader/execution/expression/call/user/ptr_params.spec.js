/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `
User function call tests for pointer parameters.
`;import { makeTestGroup } from '../../../../../../common/framework/test_group.js';
import { GPUTest } from '../../../../../gpu_test.js';

export const g = makeTestGroup(GPUTest);

function wgslTypeDecl(kind) {
  switch (kind) {
    case 'vec4i':
      return `
alias T = vec4i;
`;
    case 'array':
      return `
alias T = array<vec4f, 3>;
`;
    case 'struct':
      return `
struct S {
a : i32,
b : u32,
c : i32,
d : u32,
}
alias T = S;
`;
  }
}

function valuesForType(kind) {
  switch (kind) {
    case 'vec4i':
      return new Uint32Array([1, 2, 3, 4]);
    case 'array':
      return new Float32Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
    case 'struct':
      return new Uint32Array([1, 2, 3, 4]);
  }
}

function run(
t,
wgsl,
inputUsage,
input,
expected)
{
  const pipeline = t.device.createComputePipeline({
    layout: 'auto',
    compute: {
      module: t.device.createShaderModule({ code: wgsl }),
      entryPoint: 'main'
    }
  });

  const inputBuffer = t.makeBufferWithContents(
    input,
    inputUsage === 'uniform' ? GPUBufferUsage.UNIFORM : GPUBufferUsage.STORAGE
  );

  const outputBuffer = t.createBufferTracked({
    size: expected.buffer.byteLength,
    usage: GPUBufferUsage.STORAGE | GPUBufferUsage.COPY_SRC
  });

  const bindGroup = t.device.createBindGroup({
    layout: pipeline.getBindGroupLayout(0),
    entries: [
    { binding: 0, resource: { buffer: inputBuffer } },
    { binding: 1, resource: { buffer: outputBuffer } }]

  });

  const encoder = t.device.createCommandEncoder();
  const pass = encoder.beginComputePass();
  pass.setPipeline(pipeline);
  pass.setBindGroup(0, bindGroup);
  pass.dispatchWorkgroups(1);
  pass.end();
  t.queue.submit([encoder.finish()]);

  t.expectGPUBufferValuesEqual(outputBuffer, expected);
}

g.test('read_full_object').
desc('Test a pointer parameter can be read by a callee function').
params((u) =>
u.
combine('address_space', ['function', 'private', 'workgroup', 'storage', 'uniform']).
combine('call_indirection', [0, 1, 2]).
combine('type', ['vec4i', 'array', 'struct'])
).
fn((t) => {
  switch (t.params.address_space) {
    case 'workgroup':
    case 'storage':
    case 'uniform':
      t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');
  }

  const main = {
    function: `
@compute @workgroup_size(1)
fn main() {
  var F : T = input;
  f0(&F);
}
`,
    private: `
var<private> P : T;
@compute @workgroup_size(1)
fn main() {
  P = input;
  f0(&P);
}
`,
    workgroup: `
var<workgroup> W : T;
@compute @workgroup_size(1)
fn main() {
  W = input;
  f0(&W);
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  f0(&input);
}
`,
    uniform: `
@compute @workgroup_size(1)
fn main() {
  f0(&input);
}
`
  }[t.params.address_space];

  let call_chain = '';
  for (let i = 0; i < t.params.call_indirection; i++) {
    call_chain += `
fn f${i}(p : ptr<${t.params.address_space}, T>) {
  f${i + 1}(p);
}
`;
  }

  const inputVar =
  t.params.address_space === 'uniform' ?
  `@binding(0) @group(0) var<uniform> input : T;` :
  `@binding(0) @group(0) var<storage, read> input : T;`;

  const wgsl = `
${wgslTypeDecl(t.params.type)}

${inputVar}

@binding(1) @group(0) var<storage, read_write> output : T;

fn f${t.params.call_indirection}(p : ptr<${t.params.address_space}, T>) {
    output = *p;
}

${call_chain}

${main}
`;

  const values = valuesForType(t.params.type);

  run(t, wgsl, t.params.address_space === 'uniform' ? 'uniform' : 'storage', values, values);
});

g.test('read_ptr_to_member').
desc('Test a pointer parameter to a member of a structure can be read by a callee function').
params((u) =>
u.combine('address_space', ['function', 'private', 'workgroup', 'storage', 'uniform'])
).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const main = {
    function: `
@compute @workgroup_size(1)
fn main() {
  var v : S = input;
  output = f0(&v);
}
`,
    private: `
var<private> P : S;
@compute @workgroup_size(1)
fn main() {
  P = input;
  output = f0(&P);
}
`,
    workgroup: `
var<workgroup> W : S;
@compute @workgroup_size(1)
fn main() {
  W = input;
  output = f0(&W);
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  output = f0(&input);
}
`,
    uniform: `
@compute @workgroup_size(1)
fn main() {
  output = f0(&input);
}
`
  }[t.params.address_space];

  const inputVar =
  t.params.address_space === 'uniform' ?
  `@binding(0) @group(0) var<uniform> input : S;` :
  `@binding(0) @group(0) var<storage, read> input : S;`;

  const wgsl = `
struct S {
  a : vec4i,
  b : T,
  c : vec4i,
}

struct T {
  a : vec4i,
  b : vec4i,
}


${inputVar}
@binding(1) @group(0) var<storage, read_write> output : T;

fn f2(p : ptr<${t.params.address_space}, T>) -> T {
  return *p;
}

fn f1(p : ptr<${t.params.address_space}, S>) -> T {
  return f2(&(*p).b);
}

fn f0(p : ptr<${t.params.address_space}, S>) -> T {
  return f1(p);
}

${main}
`;


  const input = new Uint32Array([
  /* S.a */1, 2, 3, 4,
  /* S.b.a */5, 6, 7, 8,
  /* S.b.b */9, 10, 11, 12,
  /* S.c */13, 14, 15, 16]
  );


  const expected = new Uint32Array([
  /* S.b.a */5, 6, 7, 8,
  /* S.b.b */9, 10, 11, 12]
  );

  run(t, wgsl, t.params.address_space === 'uniform' ? 'uniform' : 'storage', input, expected);
});

g.test('read_ptr_to_element').
desc('Test a pointer parameter to an element of an array can be read by a callee function').
params((u) =>
u.combine('address_space', ['function', 'private', 'workgroup', 'storage', 'uniform'])
).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const main = {
    function: `
@compute @workgroup_size(1)
fn main() {
  var v : T = input;
  output = f0(&v);
}
`,
    private: `
var<private> P : T;
@compute @workgroup_size(1)
fn main() {
  P = input;
  output = f0(&P);
}
`,
    workgroup: `
var<workgroup> W : T;
@compute @workgroup_size(1)
fn main() {
  W = input;
  output = f0(&W);
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  output = f0(&input);
}
`,
    uniform: `
@compute @workgroup_size(1)
fn main() {
  output = f0(&input);
}
`
  }[t.params.address_space];

  const inputVar =
  t.params.address_space === 'uniform' ?
  `@binding(0) @group(0) var<uniform> input : T;` :
  `@binding(0) @group(0) var<storage, read> input : T;`;

  const wgsl = `
alias T3 = vec4i;
alias T2 = array<T3, 2>;
alias T1 = array<T2, 3>;
alias T = array<T1, 2>;

${inputVar}
@binding(1) @group(0) var<storage, read_write> output : T3;

fn f2(p : ptr<${t.params.address_space}, T2>) -> T3 {
  return (*p)[1];
}

fn f1(p : ptr<${t.params.address_space}, T1>) -> T3 {
  return f2(&(*p)[0]) + f2(&(*p)[2]);
}

fn f0(p : ptr<${t.params.address_space}, T>) -> T3 {
  return f1(&(*p)[0]);
}

${main}
`;


  const input = new Uint32Array([
  /* [0][0][0] */1, 2, 3, 4,
  /* [0][0][1] */5, 6, 7, 8,
  /* [0][1][0] */9, 10, 11, 12,
  /* [0][1][1] */13, 14, 15, 16,
  /* [0][2][0] */17, 18, 19, 20,
  /* [0][2][1] */21, 22, 23, 24,
  /* [1][0][0] */25, 26, 27, 28,
  /* [1][0][1] */29, 30, 31, 32,
  /* [1][1][0] */33, 34, 35, 36,
  /* [1][1][1] */37, 38, 39, 40,
  /* [1][2][0] */41, 42, 43, 44,
  /* [1][2][1] */45, 46, 47, 48]
  );
  const expected = new Uint32Array([/* [0][0][1] + [0][2][1] */5 + 21, 6 + 22, 7 + 23, 8 + 24]);

  run(t, wgsl, t.params.address_space === 'uniform' ? 'uniform' : 'storage', input, expected);
});

g.test('write_full_object').
desc('Test a pointer parameter can be written to by a callee function').
params((u) =>
u.
combine('address_space', ['function', 'private', 'workgroup', 'storage']).
combine('call_indirection', [0, 1, 2]).
combine('type', ['vec4i', 'array', 'struct'])
).
fn((t) => {
  switch (t.params.address_space) {
    case 'workgroup':
    case 'storage':
      t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');
  }

  const ptr =
  t.params.address_space === 'storage' ?
  `ptr<storage, T, read_write>` :
  `ptr<${t.params.address_space}, T>`;

  const main = {
    function: `
@compute @workgroup_size(1)
fn main() {
  var F : T;
  f0(&F);
  output = F;
}
`,
    private: `
var<private> P : T;
@compute @workgroup_size(1)
fn main() {
  f0(&P);
  output = P;
}
`,
    workgroup: `
var<workgroup> W : T;
@compute @workgroup_size(1)
fn main() {
  f0(&W);
  output = W;
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  f0(&output);
}
`
  }[t.params.address_space];

  let call_chain = '';
  for (let i = 0; i < t.params.call_indirection; i++) {
    call_chain += `
fn f${i}(p : ${ptr}) {
  f${i + 1}(p);
}
`;
  }

  const wgsl = `
${wgslTypeDecl(t.params.type)}

@binding(0) @group(0) var<uniform> input : T;
@binding(1) @group(0) var<storage, read_write> output : T;

fn f${t.params.call_indirection}(p : ${ptr}) {
  *p = input;
}

${call_chain}

${main}
`;

  const values = valuesForType(t.params.type);

  run(t, wgsl, 'uniform', values, values);
});

g.test('write_ptr_to_member').
desc(
  'Test a pointer parameter to a member of a structure can be written to by a callee function'
).
params((u) => u.combine('address_space', ['function', 'private', 'workgroup', 'storage'])).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const main = {
    function: `
@compute @workgroup_size(1)
fn main() {
  var v : S;
  f0(&v);
  output = v;
}
`,
    private: `
var<private> P : S;
@compute @workgroup_size(1)
fn main() {
  f0(&P);
  output = P;
}
`,
    workgroup: `
var<workgroup> W : S;
@compute @workgroup_size(1)
fn main() {
  f0(&W);
  output = W;
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  f1(&output);
}
`
  }[t.params.address_space];

  const ptr = (ty) =>
  t.params.address_space === 'storage' ?
  `ptr<storage, ${ty}, read_write>` :
  `ptr<${t.params.address_space}, ${ty}>`;

  const wgsl = `
struct S {
  a : vec4i,
  b : T,
  c : vec4i,
}

struct T {
  a : vec4i,
  b : vec4i,
}


@binding(0) @group(0) var<storage> input : T;
@binding(1) @group(0) var<storage, read_write> output : S;

fn f2(p : ${ptr('T')}) {
  *p = input;
}

fn f1(p : ${ptr('S')}) {
  f2(&(*p).b);
}

fn f0(p : ${ptr('S')}) {
  f1(p);
}

${main}
`;


  const input = new Uint32Array([
  /* S.b.a */5, 6, 7, 8,
  /* S.b.b */9, 10, 11, 12]
  );


  const expected = new Uint32Array([
  /* S.a   */0, 0, 0, 0,
  /* S.b.a */5, 6, 7, 8,
  /* S.b.b */9, 10, 11, 12,
  /* S.c   */0, 0, 0, 0]
  );

  run(t, wgsl, 'storage', input, expected);
});

g.test('write_ptr_to_element').
desc('Test a pointer parameter to an element of an array can be written to by a callee function').
params((u) => u.combine('address_space', ['function', 'private', 'workgroup', 'storage'])).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const main = {
    function: `
@compute @workgroup_size(1)
fn main() {
  var v : T;
  f0(&v);
  output = v;
}
`,
    private: `
var<private> P : T;
@compute @workgroup_size(1)
fn main() {
  f0(&P);
  output = P;
}
`,
    workgroup: `
var<workgroup> W : T;
@compute @workgroup_size(1)
fn main() {
  f0(&W);
  output = W;
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  f0(&output);
}
`
  }[t.params.address_space];

  const ptr = (ty) =>
  t.params.address_space === 'storage' ?
  `ptr<storage, ${ty}, read_write>` :
  `ptr<${t.params.address_space}, ${ty}>`;

  const wgsl = `
alias T3 = vec4i;
alias T2 = array<T3, 2>;
alias T1 = array<T2, 3>;
alias T = array<T1, 2>;

@binding(0) @group(0) var<storage, read> input : T3;
@binding(1) @group(0) var<storage, read_write> output : T;

fn f2(p : ${ptr('T2')}) {
  (*p)[1] = input;
}

fn f1(p : ${ptr('T1')}) {
  f2(&(*p)[0]);
  f2(&(*p)[2]);
}

fn f0(p : ${ptr('T')}) {
  f1(&(*p)[0]);
}

${main}
`;

  const input = new Uint32Array([1, 2, 3, 4]);


  const expected = new Uint32Array([
  /* [0][0][0] */0, 0, 0, 0,
  /* [0][0][1] */1, 2, 3, 4,
  /* [0][1][0] */0, 0, 0, 0,
  /* [0][1][1] */0, 0, 0, 0,
  /* [0][2][0] */0, 0, 0, 0,
  /* [0][2][1] */1, 2, 3, 4,
  /* [1][0][0] */0, 0, 0, 0,
  /* [1][0][1] */0, 0, 0, 0,
  /* [1][1][0] */0, 0, 0, 0,
  /* [1][1][1] */0, 0, 0, 0,
  /* [1][2][0] */0, 0, 0, 0,
  /* [1][2][1] */0, 0, 0, 0]
  );

  run(t, wgsl, 'storage', input, expected);
});

g.test('atomic_ptr_to_element').
desc(
  'Test a pointer parameter to an atomic<i32> of an array can be read from and written to by a callee function'
).
params((u) => u.combine('address_space', ['workgroup', 'storage'])).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const main = {
    workgroup: `
var<workgroup> W_input : T;
var<workgroup> W_output : T;
@compute @workgroup_size(1)
fn main() {
  // Copy input -> W_input
  for (var i = 0; i < 2; i++) {
    for (var j = 0; j < 3; j++) {
      for (var k = 0; k < 2; k++) {
        atomicStore(&W_input[k][j][i], atomicLoad(&input[k][j][i]));
      }
    }
  }

  f0(&W_input, &W_output);

  // Copy W_output -> output
  for (var i = 0; i < 2; i++) {
    for (var j = 0; j < 3; j++) {
      for (var k = 0; k < 2; k++) {
        atomicStore(&output[k][j][i], atomicLoad(&W_output[k][j][i]));
      }
    }
  }
}
`,
    storage: `
@compute @workgroup_size(1)
fn main() {
  f0(&input, &output);
}
`
  }[t.params.address_space];

  const ptr = (ty) =>
  t.params.address_space === 'storage' ?
  `ptr<storage, ${ty}, read_write>` :
  `ptr<${t.params.address_space}, ${ty}>`;

  const wgsl = `
alias T3 = atomic<i32>;
alias T2 = array<T3, 2>;
alias T1 = array<T2, 3>;
alias T = array<T1, 2>;

@binding(0) @group(0) var<storage, read_write> input : T;
@binding(1) @group(0) var<storage, read_write> output : T;

fn f2(in : ${ptr('T2')}, out : ${ptr('T2')}) {
  let v = atomicLoad(&(*in)[0]);
  atomicStore(&(*out)[1], v);
}

fn f1(in : ${ptr('T1')}, out : ${ptr('T1')}) {
  f2(&(*in)[0], &(*out)[1]);
  f2(&(*in)[2], &(*out)[0]);
}

fn f0(in : ${ptr('T')}, out : ${ptr('T')}) {
  f1(&(*in)[1], &(*out)[0]);
}

${main}
`;


  const input = new Uint32Array([
  /* [0][0][0] */1,
  /* [0][0][1] */2,
  /* [0][1][0] */3,
  /* [0][1][1] */4,
  /* [0][2][0] */5,
  /* [0][2][1] */6,
  /* [1][0][0] */7, // -> [0][1][1]
  /* [1][0][1] */8,
  /* [1][1][0] */9,
  /* [1][1][1] */10,
  /* [1][2][0] */11, // -> [0][0][1]
  /* [1][2][1] */12]
  );


  const expected = new Uint32Array([
  /* [0][0][0] */0,
  /* [0][0][1] */11,
  /* [0][1][0] */0,
  /* [0][1][1] */7,
  /* [0][2][0] */0,
  /* [0][2][1] */0,
  /* [1][0][0] */0,
  /* [1][0][1] */0,
  /* [1][1][0] */0,
  /* [1][1][1] */0,
  /* [1][2][0] */0,
  /* [1][2][1] */0]
  );

  run(t, wgsl, 'storage', input, expected);
});

g.test('array_length').
desc(
  'Test a pointer parameter to a runtime sized array can be used by arrayLength() in a callee function'
).
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const wgsl = `
@binding(0) @group(0) var<storage, read> arr : array<u32>;
@binding(1) @group(0) var<storage, read_write> output : u32;

fn f2(p : ptr<storage, array<u32>, read>) -> u32 {
  return arrayLength(p);
}

fn f1(p : ptr<storage, array<u32>, read>) -> u32 {
  return f2(p);
}

fn f0(p : ptr<storage, array<u32>, read>) -> u32 {
  return f1(p);
}

@compute @workgroup_size(1)
fn main() {
  output = f0(&arr);
}
`;

  const input = new Uint32Array([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);
  const expected = new Uint32Array([12]);

  run(t, wgsl, 'storage', input, expected);
});

g.test('mixed_ptr_parameters').
desc('Test that functions can accept multiple, mixed pointer parameters').
fn((t) => {
  t.skipIfLanguageFeatureNotSupported('unrestricted_pointer_parameters');

  const wgsl = `
@binding(0) @group(0) var<uniform> input : array<vec4i, 4>;
@binding(1) @group(0) var<storage, read_write> output : array<vec4i, 4>;

fn sum(f : ptr<function, i32>,
       w : ptr<workgroup, atomic<i32>>,
       p : ptr<private, i32>,
       u : ptr<uniform, vec4i>) -> vec4i {

  return vec4(*f + atomicLoad(w) + *p) + *u;
}

struct S {
  i : i32,
}

var<private> P0 = S(0);
var<private> P1 = S(10);
var<private> P2 = 20;
var<private> P3 = 30;

struct T {
  i : atomic<i32>,
}

var<workgroup> W0 : T;
var<workgroup> W1 : atomic<i32>;
var<workgroup> W2 : T;
var<workgroup> W3 : atomic<i32>;

@compute @workgroup_size(1)
fn main() {
  atomicStore(&W0.i, 0);
  atomicStore(&W1,   100);
  atomicStore(&W2.i, 200);
  atomicStore(&W3,   300);

  var F = array(0, 1000, 2000, 3000);

  output[0] = sum(&F[2], &W3,   &P1.i, &input[0]); // vec4(2310) + vec4(1, 2, 3, 4)
  output[1] = sum(&F[1], &W2.i, &P0.i, &input[1]); // vec4(1200) + vec4(4, 3, 2, 1)
  output[2] = sum(&F[3], &W0.i, &P3,   &input[2]); // vec4(3030) + vec4(2, 4, 1, 3)
  output[3] = sum(&F[2], &W1,   &P2,   &input[3]); // vec4(2120) + vec4(4, 1, 2, 3)
}
`;


  const input = new Uint32Array([
  /* [0] */1, 2, 3, 4,
  /* [1] */4, 3, 2, 1,
  /* [2] */2, 4, 1, 3,
  /* [3] */4, 1, 2, 3]
  );


  const expected = new Uint32Array([
  /* [0] */2311, 2312, 2313, 2314,
  /* [1] */1204, 1203, 1202, 1201,
  /* [2] */3032, 3034, 3031, 3033,
  /* [3] */2124, 2121, 2122, 2123]
  );

  run(t, wgsl, 'uniform', input, expected);
});