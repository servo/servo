/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `Validation tests for semicolon placements`;import { makeTestGroup } from '../../../../common/framework/test_group.js';
import { ShaderValidationTest } from '../shader_validation_test.js';

export const g = makeTestGroup(ShaderValidationTest);

g.test('module_scope_single').
desc(`Test that a semicolon can be placed at module scope.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `;`);
});

g.test('module_scope_multiple').
desc(`Test that multiple semicolons can be placed at module scope.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `;;;`);
});

g.test('after_enable').
desc(`Test that a semicolon must be placed after an enable directive.`).
beforeAllSubcases((t) => {
  t.selectDeviceOrSkipTestCase({ requiredFeatures: ['shader-f16'] });
}).
fn((t) => {
  t.expectCompileResult( /* pass */true, `enable f16;`);
  t.expectCompileResult( /* pass */false, `enable f16`);
});

g.test('after_struct_decl').
desc(`Test that a semicolon can be placed after an struct declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `struct S { x : i32 };`);
  t.expectCompileResult( /* pass */true, `struct S { x : i32 }`);
});

g.test('after_member').
desc(`Test that a semicolon must not be placed after an struct member declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `struct S { x : i32 }`);
  t.expectCompileResult( /* pass */false, `struct S { x : i32; }`);
});

g.test('after_func_decl').
desc(`Test that a semicolon can be placed after a function declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() {};`);
  t.expectCompileResult( /* pass */true, `fn f() {}`);
});

g.test('after_type_alias_decl').
desc(`Test that a semicolon must be placed after an type alias declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `alias T = i32;`);
  t.expectCompileResult( /* pass */false, `alias T = i32`);
});

g.test('after_return').
desc(`Test that a semicolon must be placed after a return statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { return; }`);
  t.expectCompileResult( /* pass */false, `fn f() { return }`);
});

g.test('after_call').
desc(`Test that a semicolon must be placed after a function call.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { workgroupBarrier(); }`);
  t.expectCompileResult( /* pass */false, `fn f() { workgroupBarrier() }`);
});

g.test('after_module_const_decl').
desc(`Test that a semicolon must be placed after a module-scope const declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `const v = 1;`);
  t.expectCompileResult( /* pass */false, `const v = 1`);
});

g.test('after_fn_const_decl').
desc(`Test that a semicolon must be placed after a function-scope const declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { const v = 1; }`);
  t.expectCompileResult( /* pass */false, `fn f() { const v = 1 }`);
});

g.test('after_module_var_decl').
desc(`Test that a semicolon must be placed after a module-scope var declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `var<private> v = 1;`);
  t.expectCompileResult( /* pass */false, `var<private> v = 1`);
});

g.test('after_fn_var_decl').
desc(`Test that a semicolon must be placed after a function-scope var declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { var v = 1; }`);
  t.expectCompileResult( /* pass */false, `fn f() { var v = 1 }`);
});

g.test('after_let_decl').
desc(`Test that a semicolon must be placed after a let declaration.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { let v = 1; }`);
  t.expectCompileResult( /* pass */false, `fn f() { let v = 1 }`);
});

g.test('after_discard').
desc(`Test that a semicolon must be placed after a discard statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { discard; }`);
  t.expectCompileResult( /* pass */false, `fn f() { discard }`);
});

g.test('after_assignment').
desc(`Test that a semicolon must be placed after an assignment statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { var v = 1; v = 2; }`);
  t.expectCompileResult( /* pass */false, `fn f() { var v = 1; v = 2 }`);
});

g.test('after_fn_const_assert').
desc(`Test that a semicolon must be placed after an function-scope static assert.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { const_assert(true); }`);
  t.expectCompileResult( /* pass */false, `fn f() { const_assert(true) }`);
});

g.test('function_body_single').
desc(`Test that a semicolon can be placed in a function body.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { ; }`);
});

g.test('function_body_multiple').
desc(`Test that multiple semicolons can be placed in a function body.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { ;;; }`);
});

g.test('compound_statement_single').
desc(`Test that a semicolon can be placed in a compound statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { { ; } }`);
});

g.test('compound_statement_multiple').
desc(`Test that multiple semicolons can be placed in a compound statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { { ;;; } }`);
});

g.test('after_compound_statement').
desc(`Test that a semicolon can be placed after a compound statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { {} ; }`);
});

g.test('after_if').
desc(`Test that a semicolon can be placed after an if-statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { if true {} ; }`);
});

g.test('after_if_else').
desc(`Test that a semicolon can be placed after an if-else-statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { if true {} else {} ; }`);
});

g.test('after_switch').
desc(`Test that a semicolon can be placed after an switch-statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { switch 1 { default {} } ; }`);
});

g.test('after_case').
desc(`Test that a semicolon cannot be placed after a non-default switch case.`).
fn((t) => {
  t.expectCompileResult( /* pass */false, `fn f() { switch 1 { case 1 {}; default {} } }`);
  t.expectCompileResult( /* pass */true, `fn f() { switch 1 { case 1 {} default {} } }`);
});

g.test('after_case_break').
desc(`Test that a semicolon must be placed after a case break statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */false, `fn f() { switch 1 { case 1 { break } default {} } }`);
  t.expectCompileResult( /* pass */true, `fn f() { switch 1 { case 1 { break; } default {} } }`);
});

g.test('after_default_case').
desc(`Test that a semicolon cannot be placed after a default switch case.`).
fn((t) => {
  t.expectCompileResult( /* pass */false, `fn f() { switch 1 { default {}; } }`);
  t.expectCompileResult( /* pass */true, `fn f() { switch 1 { default {} } }`);
});

g.test('after_default_case_break').
desc(`Test that a semicolon cannot be placed after a default switch case.`).
fn((t) => {
  t.expectCompileResult( /* pass */false, `fn f() { switch 1 { default { break } } }`);
  t.expectCompileResult( /* pass */true, `fn f() { switch 1 { default { break; } } }`);
});

g.test('after_for').
desc(`Test that a semicolon can be placed after a for-loop.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { for (; false;) {}; }`);
});

g.test('after_for_break').
desc(`Test that a semicolon must be placed after a for-loop break statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { for (; false;) { break; } }`);
  t.expectCompileResult( /* pass */false, `fn f() { for (; false;) { break } }`);
});

g.test('after_loop').
desc(`Test that a semicolon can be placed after a loop.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { loop { break; }; }`);
});

g.test('after_loop_break').
desc(`Test that a semicolon must be placed after a loop break statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { loop { break; }; }`);
  t.expectCompileResult( /* pass */false, `fn f() { loop { break }; }`);
});

g.test('after_loop_break_if').
desc(`Test that a semicolon must be placed after a loop break-if statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { loop { continuing { break if true; } }; }`);
  t.expectCompileResult( /* pass */false, `fn f() { loop { continuing { break if true } }; }`);
});

g.test('after_loop_continue').
desc(`Test that a semicolon must be placed after a loop continue statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { loop { if true { continue; } { break; } } }`);
  t.expectCompileResult( /* pass */false, `fn f() { loop { if true { continue } { break; } } }`);
});

g.test('after_continuing').
desc(`Test that a semicolon cannot be placed after a continuing.`).
fn((t) => {
  t.expectCompileResult( /* pass */false, `fn f() { loop { break; continuing{}; } }`);
  t.expectCompileResult( /* pass */true, `fn f() { loop { break; continuing{} } }`);
});

g.test('after_while').
desc(`Test that a semicolon cannot be placed after a while-loop.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { while false {}; }`);
});

g.test('after_while_break').
desc(`Test that a semicolon must be placed after a while break statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { while false { break; } }`);
  t.expectCompileResult( /* pass */false, `fn f() { while false { break } }`);
});

g.test('after_while_continue').
desc(`Test that a semicolon must be placed after a while continue statement.`).
fn((t) => {
  t.expectCompileResult( /* pass */true, `fn f() { while false { continue; } }`);
  t.expectCompileResult( /* pass */false, `fn f() { while false { continue } }`);
});