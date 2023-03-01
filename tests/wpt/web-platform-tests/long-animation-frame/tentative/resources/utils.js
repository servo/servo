const windowLoaded = new Promise(resolve => window.addEventListener('load', resolve));
setup(() =>
  assert_implements(window.PerformanceLongAnimationFrameTiming,
    'Long animation frames are not supported.'));

const very_long_frame_duration = 360;

function loaf_promise() {
  return new Promise(resolve => {
      const observer = new PerformanceObserver(entries => {
          const entry = entries.getEntries()[0];
          if (entry.duration >= very_long_frame_duration)
            resolve(entry);
      });

      observer.observe({entryTypes: ['long-animation-frame']});
  });
}

const no_long_frame_timeout = very_long_frame_duration * 2;

function busy_wait(ms_delay = very_long_frame_duration) {
  const deadline = performance.now() + ms_delay;
  while (performance.now() < deadline) {}
}

async function expect_long_frame(cb, t) {
  await windowLoaded;
  await new Promise(resolve => t.step_timeout(resolve, 0));
  const timeout = new Promise((resolve, reject) =>
    t.step_timeout(() => reject("timeout"), no_long_frame_timeout));
  const receivedLongFrame = loaf_promise();
  await cb();
  const entry = await Promise.race([
    receivedLongFrame,
    timeout
  ]);
  return entry;
}

async function expect_no_long_frame(cb, t) {
  await windowLoaded;
  for (let i = 0; i < 5; ++i) {
    const receivedLongFrame = loaf_promise();
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
  return new RemoteContext(uuid);
}


async function prepare_exec_popup(t, origin) {
  const url = new URL("/common/dispatcher/remote-executor.html", origin);
  const uuid = token();
  url.searchParams.set("uuid", uuid);
  const popup = window.open(url);
  t.add_cleanup(() => popup.close());
  return new RemoteContext(uuid);
}
