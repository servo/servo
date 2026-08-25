// Worker side of the LoAF congested-moment test. A congested moment is
// surfaced as a `long-animation-frame` entry inside the worker.
importScripts('utils-worker.js');

// A single task well over the 200ms congestion threshold.
const LONG_TASK_DURATION_MS = 250;

// Each flood task runs longer than the 5ms per-script threshold (so it is
// attributed in `scripts`) but well under the 200ms congestion threshold.
// Together they saturate the loop past the threshold.
const FLOOD_TASK_COUNT = 20;
const FLOOD_TASK_DURATION_MS = 15;

// Idle gap used by the drainer timer. It must exceed a single task's duration
// so the queue is guaranteed to drain (go empty) before the timer fires, which
// is the signal that ends a saturated interval. The drainer is posted from the
// last backlogged task, so it only needs to outlast that one task.
const DRAIN_GAP_MS = 100;

// A flood long enough to stay saturated well past the 200ms threshold. A
// congested moment is the whole saturated interval, so this is used to prove
// that even a long saturation is surfaced as a single entry covering the whole
// interval.
const FLOOD_LONG_TASK_COUNT = 45;

// Delay before the second flood in the two-floods scenario. It must be long
// enough that the first flood (FLOOD_TASK_COUNT tasks) has fully drained before
// the second one starts, so a real idle gap separates the two congested
// moments.
const TWO_FLOODS_GAP_MS = 600;

// Negative case: a steady async iteration where each task posts the next only
// after finishing, so backlog depth stays at 1. The total work exceeds the
// 200ms threshold, but with no queuing delay it must not be reported as a
// congested moment.
const NO_CONGESTION_TASK_COUNT = 20;

// Resolves once `target_count` congested-moment entries (duration >= the 200ms
// threshold) have been observed, returning them as an array.
function observe_congested_moments(target_count) {
  return new Promise((resolve) => {
    const collected = [];
    const observer = new PerformanceObserver((entries, obs) => {
      for (const entry of entries.getEntries()) {
        if (entry.duration >= 200) {
          collected.push(entry);
        }
      }
      if (collected.length >= target_count) {
        obs.disconnect();
        resolve(collected);
      }
    });
    observer.observe({type: 'long-animation-frame', buffered: true});
  });
}

// Long-task scenario: a long task that backlogs the queue. A lone long task is
// not "congestion" under the queuing-delay model, so we enqueue a trailing task
// alongside the long task up front. While the long task blocks the loop, the
// trailing task sits backlogged in the queue, which is the queuing delay the
// congested moment reports.
function run_long_task() {
  const channel = new MessageChannel();
  const durations = [LONG_TASK_DURATION_MS, FLOOD_TASK_DURATION_MS];
  let i = 0;
  channel.port1.onmessage = () => {
    busy_wait(durations[i++]);
    if (i === durations.length) {
      // Both tasks have run and the queue is empty. Schedule a timer drainer so
      // an idle gap follows, which ends the saturated interval and reports it.
      post_drainer();
    }
  };
  channel.port1.start();
  // Post both tasks synchronously so the trailing task is enqueued before the
  // long task starts running (backlog depth >= 2).
  channel.port2.postMessage(0);
  channel.port2.postMessage(0);
}

// Schedules an empty timer task after an idle gap (DRAIN_GAP_MS). The idle gap
// makes the queue drain, which is what closes and reports the congested moment
// (a plain message would run back-to-back and be folded into the moment instead
// of ending it). This lets the test finalize the interval without waiting for
// worker shutdown.
function post_drainer() {
  setTimeout(() => {}, DRAIN_GAP_MS);
}

// Flood helper (used by the flood, flood-long, and two-floods scenarios): a
// flood of short tasks that are all enqueued before the first one starts
// running. Because every task is scheduled before its predecessor begins
// (backlog depth >= 2), the queue stays backlogged, which is the queuing delay
// the congested-moment detection reports. Each task runs a short busy-wait so
// the run stays saturated past the threshold.
function run_task_flood(count) {
  const channel = new MessageChannel();
  let run = 0;
  channel.port1.onmessage = () => {
    busy_wait(FLOOD_TASK_DURATION_MS);
    if (++run === count) {
      // The whole flood has run and the queue is empty. Schedule a timer
      // drainer so an idle gap follows, ending the saturated interval and
      // reporting it.
      post_drainer();
    }
  };
  channel.port1.start();
  for (let i = 0; i < count; i++) {
    channel.port2.postMessage(0);
  }
}

// Negative case: steady async iteration. Each task posts the next one only
// after it finishes, so at most one task is queued at a time (backlog depth 1)
// and there is no queuing delay. Resolves once all `count` tasks have run.
function run_async_iteration(count) {
  return new Promise((resolve) => {
    const channel = new MessageChannel();
    let remaining = count;
    channel.port1.onmessage = () => {
      busy_wait(FLOOD_TASK_DURATION_MS);
      if (--remaining > 0) {
        channel.port2.postMessage(0);
      } else {
        resolve();
      }
    };
    channel.port1.start();
    channel.port2.postMessage(0);
  });
}

function serialize_entry(entry) {
  return {
    entryType: entry.entryType,
    startTime: entry.startTime,
    duration: entry.duration,
    scriptCount: entry.scriptCount,
    scripts: (entry.scripts ?? []).map((s) => ({
                                         invoker: s.invoker,
                                         sourceURL: s.sourceURL,
                                       })),
  };
}

self.onmessage = async (e) => {
  if (e.data === 'long-task') {
    const congested = observe_congested_moments(1);
    run_long_task();
    const [entry] = await congested;
    self.postMessage(serialize_entry(entry));
  } else if (e.data === 'flood') {
    const congested = observe_congested_moments(1);
    run_task_flood(FLOOD_TASK_COUNT);
    const [entry] = await congested;
    self.postMessage(serialize_entry(entry));
  } else if (e.data === 'flood-long') {
    const congested = observe_congested_moments(1);
    run_task_flood(FLOOD_LONG_TASK_COUNT);
    const entries = await congested;
    self.postMessage({entries: entries.map(serialize_entry)});
  } else if (e.data === 'two-floods') {
    // Two separate floods separated by a real idle gap must be reported as two
    // distinct, non-overlapping congested moments. This is what the idle-gap
    // boundary (a moment closes only when the queue actually drains)
    // guarantees.
    const congested = observe_congested_moments(2);
    run_task_flood(FLOOD_TASK_COUNT);
    setTimeout(() => run_task_flood(FLOOD_TASK_COUNT), TWO_FLOODS_GAP_MS);
    const entries = await congested;
    self.postMessage({entries: entries.map(serialize_entry)});
  } else if (e.data === 'no-congestion') {
    // Run a steady async iteration, then report how many congested moments were
    // recorded. Under the queuing-delay model this must be zero.
    await run_async_iteration(NO_CONGESTION_TASK_COUNT);
    const congested = performance.getEntriesByType('long-animation-frame')
                          .filter((entry) => entry.duration >= 200);
    self.postMessage({congestedCount: congested.length});
  }
};
