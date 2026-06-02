// META: global=window,worker
// META: script=../dom/abort/resources/abort-signal-any-tests.js

abortSignalAnySignalOnlyTests(TaskSignal);
abortSignalAnyTests(TaskSignal, AbortController);
abortSignalAnyTests(TaskSignal, TaskController);
