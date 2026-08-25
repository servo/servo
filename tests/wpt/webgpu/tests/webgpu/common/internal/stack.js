/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/ // Returns the stack trace of an Error, but without the extra boilerplate at the bottom
// (e.g. RunCaseSpecific, processTicksAndRejections, etc.), for logging.
export function extractImportantStackTrace(e) {let stack = e.stack;if (!stack) {
    return '';
  }
  const redundantMessage = 'Error: ' + e.message + '\n';
  if (stack.startsWith(redundantMessage)) {
    stack = stack.substring(redundantMessage.length);
  }

  const lines = stack.split('\n');
  for (let i = lines.length - 1; i >= 0; --i) {
    const line = lines[i];
    if (line.indexOf('.spec.') !== -1) {
      return lines.slice(0, i + 1).join('\n');
    }
  }
  return stack;
}

// *** Examples ***
//
// Node fail()
// > Error:
// >    at CaseRecorder.fail (/Users/kainino/src/cts/src/common/framework/logger.ts:99:30)
// >    at RunCaseSpecific.exports.g.test.t [as fn] (/Users/kainino/src/cts/src/unittests/logger.spec.ts:80:7)
// x    at RunCaseSpecific.run (/Users/kainino/src/cts/src/common/framework/test_group.ts:121:18)
// x    at processTicksAndRejections (internal/process/task_queues.js:86:5)
//
// Node throw
// > Error: hello
// >     at RunCaseSpecific.g.test.t [as fn] (/Users/kainino/src/cts/src/unittests/test_group.spec.ts:51:11)
// x     at RunCaseSpecific.run (/Users/kainino/src/cts/src/common/framework/test_group.ts:121:18)
// x     at processTicksAndRejections (internal/process/task_queues.js:86:5)
//
// Firefox fail()
// > fail@http://localhost:8080/out/framework/logger.js:104:30
// > expect@http://localhost:8080/out/framework/default_fixture.js:59:16
// > @http://localhost:8080/out/unittests/util.spec.js:35:5
// x run@http://localhost:8080/out/framework/test_group.js:119:18
//
// Firefox throw
// > @http://localhost:8080/out/unittests/test_group.spec.js:48:11
// x run@http://localhost:8080/out/framework/test_group.js:119:18
//
// Safari fail()
// > fail@http://localhost:8080/out/framework/logger.js:104:39
// > expect@http://localhost:8080/out/framework/default_fixture.js:59:20
// > http://localhost:8080/out/unittests/util.spec.js:35:11
// x http://localhost:8080/out/framework/test_group.js:119:20
// x asyncFunctionResume@[native code]
// x [native code]
// x promiseReactionJob@[native code]
//
// Safari throw
// > http://localhost:8080/out/unittests/test_group.spec.js:48:20
// x http://localhost:8080/out/framework/test_group.js:119:20
// x asyncFunctionResume@[native code]
// x [native code]
// x promiseReactionJob@[native code]
//
// Chrome fail()
// x Error
// x     at CaseRecorder.fail (http://localhost:8080/out/framework/logger.js:104:30)
// x     at DefaultFixture.expect (http://localhost:8080/out/framework/default_fixture.js:59:16)
// >     at RunCaseSpecific.fn (http://localhost:8080/out/unittests/util.spec.js:35:5)
// x     at RunCaseSpecific.run (http://localhost:8080/out/framework/test_group.js:119:18)
// x     at async runCase (http://localhost:8080/out/runtime/standalone.js:37:17)
// x     at async http://localhost:8080/out/runtime/standalone.js:102:7
//
// Chrome throw
// x Error: hello
// >     at RunCaseSpecific.fn (http://localhost:8080/out/unittests/test_group.spec.js:48:11)
// x     at RunCaseSpecific.run (http://localhost:8080/out/framework/test_group.js:119:18)"
// x     at async Promise.all (index 0)
// x     at async TestGroupTest.run (http://localhost:8080/out/unittests/test_group_test.js:6:5)
// x     at async RunCaseSpecific.fn (http://localhost:8080/out/unittests/test_group.spec.js:53:15)
// x     at async RunCaseSpecific.run (http://localhost:8080/out/framework/test_group.js:119:7)
// x     at async runCase (http://localhost:8080/out/runtime/standalone.js:37:17)
// x     at async http://localhost:8080/out/runtime/standalone.js:102:7