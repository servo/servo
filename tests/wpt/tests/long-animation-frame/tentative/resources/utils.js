const windowLoaded = new Promise(resolve => window.addEventListener('load', resolve));
if ("setup" in globalThis) {
  setup(() =>
    assert_implements(window.PerformanceLongAnimationFrameTiming,
      'Long animation frames are not supported.'));
}

const very_long_frame_duration = 360;
const no_long_frame_timeout = very_long_frame_duration * 2;
const waiting_for_long_frame_timeout = very_long_frame_duration * 10;

function loaf_promise(t) {
  return new Promise(resolve => {
      const observer = new PerformanceObserver(entries => {
          const entry = entries.getEntries()[0];
          // TODO: understand why we need this 5ms epsilon.
          if (entry.duration > very_long_frame_duration - 5) {
            observer.disconnect();
            resolve(entry);
          }
      });

      t.add_cleanup(() => observer.disconnect());

      observer.observe({entryTypes: ['long-animation-frame']});
  });
}

function busy_wait(ms_delay = very_long_frame_duration) {
  const deadline = performance.now() + ms_delay;
  while (performance.now() < deadline) {}
}

async function expect_long_frame(cb, t) {
  await windowLoaded;
  await new Promise(resolve => t.step_timeout(resolve, 0));
  const timeout = new Promise((resolve, reject) =>
    t.step_timeout(() => resolve("timeout"), waiting_for_long_frame_timeout));
  const receivedLongFrame = loaf_promise(t);
  await cb(t);
  const entry = await Promise.race([
    receivedLongFrame,
    timeout
  ]);
  return entry;
}

async function expect_long_frame_with_script(cb, predicate, t) {
  for (let i = 0; i < 10; ++i) {
      const entry = await expect_long_frame(cb, t);
      if (entry === "timeout" || !entry.scripts.length)
        continue;
      for (const script of entry.scripts) {
        if (predicate(script, entry))
          return [entry, script];
      }
  }

  return [];
}

async function expect_no_long_frame(cb, t) {
  await windowLoaded;
  for (let i = 0; i < 5; ++i) {
    const receivedLongFrame = loaf_promise(t);
    await cb();
    const result = await Promise.race([receivedLongFrame,
        new Promise(resolve => t.step_timeout(() => resolve("timeout"),
        no_long_frame_timeout))]);
    if (result === "timeout")
      return false;
  }

  throw new Error("Consistently creates long frame");
}

async function prepare_exec_iframe(t, origin) {
  const iframe = document.createElement("iframe");
  t.add_cleanup(() => iframe.remove());
  const url = new URL("/common/dispatcher/remote-executor.html", origin);
  const uuid = token();
  url.searchParams.set("uuid", uuid);
  iframe.src = url.href;
  document.body.appendChild(iframe);
  await new Promise(resolve => iframe.addEventListener("load", resolve));
  return [new RemoteContext(uuid), iframe];
}


async function prepare_exec_popup(t, origin) {
  const url = new URL("/common/dispatcher/remote-executor.html", origin);
  const uuid = token();
  url.searchParams.set("uuid", uuid);
  const popup = window.open(url);
  t.add_cleanup(() => popup.close());
  return [new RemoteContext(uuid), popup];
}
function test_loaf_script(cb, invoker, invokerType, label) {
  promise_test(async t => {
    let [entry, script] = [];
    [entry, script] = await expect_long_frame_with_script(cb,
      script => (
        script.invokerType === invokerType &&
        script.invoker.startsWith(invoker) &&
        script.duration >= very_long_frame_duration), t);

    assert_true(!!entry, "Entry detected");
    assert_greater_than_equal(script.duration, very_long_frame_duration);
    assert_greater_than_equal(entry.duration, script.duration);
    assert_greater_than_equal(script.executionStart, script.startTime);
    assert_greater_than_equal(script.startTime, entry.startTime)
    assert_equals(script.window, window);
    assert_equals(script.forcedStyleAndLayoutDuration, 0);
    assert_equals(script.windowAttribution, "self");
}, `LoAF script: ${invoker} ${invokerType},${label ? ` ${label}` : ''}`);

}

function test_self_user_callback(cb, invoker, label) {
    test_loaf_script(cb, invoker, "user-callback", label);
}

function test_self_event_listener(cb, invoker) {
  test_loaf_script(cb, invoker, "event-listener");
}

function test_promise_script(cb, resolve_or_reject, invoker, label) {
  test_loaf_script(cb, invoker, `${resolve_or_reject}-promise`, label);
}

function test_self_script_block(cb, invoker, type) {
  test_loaf_script(cb, invoker, type);
}
