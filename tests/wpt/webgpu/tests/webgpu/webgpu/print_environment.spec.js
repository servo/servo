/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/export const description = `

Examples of writing CTS tests with various features.

Start here when looking for examples of basic framework usage.
`;import { getResourcePath } from '../common/framework/resources.js';
import { globalTestConfig } from '../common/framework/test_config.js';
import { makeTestGroup } from '../common/framework/test_group.js';
import { getDefaultRequestAdapterOptions } from '../common/util/navigator_gpu.js';

import { GPUTest } from './gpu_test.js';

export const g = makeTestGroup(GPUTest);

/** console.log is disallowed by WPT. Work around it when we're not in WPT. */
function consoleLogIfNotWPT(x) {
  if (!('step_timeout' in globalThis)) {
    const cons = console;
    cons.log(x);
  }
}

g.test('info').
desc(
  `Test which prints what global scope (e.g. worker type) it's running in, and info about
the adapter and device that it gets.

Note, other tests should not check the global scope type to detect features; instead, they should
check for the presence of the feature they need (like HTMLCanvasElement) and skip if not available.

Run this test under various configurations to see different results
(Window, worker scopes, Node, etc.)

NOTE: If your test runtime elides logs when tests pass, you won't see the prints from this test
in the logs. On non-WPT runtimes, it will also print to the console with console.log.
WPT disallows console.log and doesn't support logs on passing tests, so this does nothing on WPT.`
).
fn((t) => {
  // `t.device` will be the default device, because no additional capabilities were requested.
  const defaultDeviceProperties = Object.fromEntries(
    function* () {
      const device = t.device;
      for (const key in device) {
        // Skip things that we don't want to JSON.stringify.
        if (['lost', 'queue', 'onuncapturederror', 'label'].includes(key)) {
          continue;
        }
        yield [key, device[key]];
      }
    }()
  );

  const info = JSON.stringify(
    {
      userAgent: globalThis.navigator?.userAgent,
      globalScope: Object.getPrototypeOf(globalThis).constructor.name,
      globalTestConfig,
      baseResourcePath: getResourcePath(''),
      defaultRequestAdapterOptions: getDefaultRequestAdapterOptions(),
      // Print all of the properties of the adapter and defaultDeviceProperties. JSON.stringify
      // will skip methods (e.g. adapter.requestDevice), because they're not stringifiable.
      adapter: t.adapter,
      defaultDevice: defaultDeviceProperties
    },
    // - Replace `undefined` with `null`.
    // - Expand iterable things into arrays.
    // - Flatten objects with prototype chains into plain objects, using `for..in`. (Otherwise,
    //   properties from the prototype chain will be ignored and things will print as `{}`.)
    (_key, value) => {
      if (value === undefined || value === null) return null;
      if (typeof value !== 'object') return value;
      if (value instanceof Array) return value;
      if (Symbol.iterator in value) return Array.from(value);

      const valueObj = value;
      return Object.fromEntries(
        function* () {
          for (const key in valueObj) {
            yield [key, valueObj[key]];
          }
        }()
      );
    },
    2
  );

  t.info(info);
  consoleLogIfNotWPT(info);
});