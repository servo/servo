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
  `Test which prints what global scope (e.g. worker type) it's running in.
Typically, tests will check for the presence of the feature they need (like HTMLCanvasElement)
and skip if it's not available.

Run this test under various configurations to see different results
(Window, worker scopes, Node, etc.)

NOTE: If your test runtime elides logs when tests pass, you won't see the prints from this test
in the logs. On non-WPT runtimes, it will also print to the console with console.log.
WPT disallows console.log and doesn't support logs on passing tests, so this does nothing on WPT.`
).
fn((t) => {
  const isCompatibilityMode = t.adapter.
  isCompatibilityMode;

  const info = JSON.stringify(
    {
      userAgent: navigator.userAgent,
      globalScope: Object.getPrototypeOf(globalThis).constructor.name,
      globalTestConfig,
      baseResourcePath: getResourcePath(''),
      defaultRequestAdapterOptions: getDefaultRequestAdapterOptions(),
      adapter: {
        isFallbackAdapter: t.adapter.isFallbackAdapter,
        isCompatibilityMode,
        info: t.adapter.info,
        features: Array.from(t.adapter.features),
        limits: t.adapter.limits
      }
    },
    // Flatten objects with prototype chains into plain objects, using `for..in`. (Otherwise,
    // properties from the prototype chain will be ignored and things will print as `{}`.)
    (_key, value) => {
      if (value === undefined || value === null) return null;
      if (typeof value !== 'object') return value;
      if (value instanceof Array) return value;

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