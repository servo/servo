/**
* AUTO-GENERATED - DO NOT EDIT. Source: https://github.com/gpuweb/cts
**/const shutdownTasks = []; /**
 * Register a callback to be run during program shutdown (triggered, e.g., by the 'beforeunload'
 * event when the webpage is being closed).
 *
 * Note such tasks should be synchronous functions; otherwise, they probably won't complete.
 */
export function registerShutdownTask(task) {
  shutdownTasks.push(task);
}

/** Run all shutdown tasks. Should only be called during program shutdown. */
export function runShutdownTasks() {
  for (const task of shutdownTasks) {
    task();
  }
  shutdownTasks.length = 0;
}