import {IdleManager, IdleManagerError, IdleManagerReceiver} from '/gen/third_party/blink/public/mojom/idle/idle_manager.mojom.m.js';

/**
 * This is a testing framework that enables us to test the user idle detection
 * by intercepting the connection between the renderer and the browser and
 * exposing a mocking API for tests.
 *
 * Usage:
 *
 * 1) Include <script src="mock.js"></script> in your file.
 * 2) Set expectations
 *   expect(addMonitor).andReturn((threshold, monitorPtr, callback) => {
 *     // mock behavior
 *   })
 * 3) Call navigator.idle.query()
 *
 * The mocking API is blink agnostic and is designed such that other engines
 * could implement it too. Here are the symbols that are exposed to tests:
 *
 * - function addMonitor(): the main/only function that can be mocked.
 * - function expect(): the main/only function that enables us to mock it.
 * - function close(): disconnects the interceptor.
 * - enum UserIdleState {IDLE, ACTIVE}: blink agnostic constants.
 * - enum ScreenIdleState {LOCKED, UNLOCKED}: blink agnostic constants.
 */

class FakeIdleMonitor {
  addMonitor(threshold, monitorPtr, callback) {
    return this.handler.addMonitor(threshold, monitorPtr);
  }
  setHandler(handler) {
    this.handler = handler;
    return this;
  }
  setBinding(binding) {
    this.binding = binding;
    return this;
  }
  close() {
    this.binding.$.close();
  }
}

self.IdleDetectorError = {};

self.addMonitor = function addMonitor(threshold, monitorPtr, callback) {
  throw new Error("expected to be overriden by tests");
}

async function close() {
  interceptor.close();
}

self.expect = function(call) {
  return {
    andReturn(callback) {
      let handler = {};
      handler[call.name] = callback;
      interceptor.setHandler(handler);
    }
  };
};

function intercept() {
  let result = new FakeIdleMonitor();

  let binding = new IdleManagerReceiver(result);
  let interceptor = new MojoInterfaceInterceptor(IdleManager.$interfaceName);
  interceptor.oninterfacerequest = e => binding.$.bindHandle(e.handle);
  interceptor.start();

  self.IdleDetectorError.SUCCESS = IdleManagerError.kSuccess;
  self.IdleDetectorError.PERMISSION_DISABLED =
      IdleManagerError.kPermissionDisabled;

  result.setBinding(binding);
  return result;
}

const interceptor = intercept();
