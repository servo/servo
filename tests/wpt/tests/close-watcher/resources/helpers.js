// TODO(domenic): consider using these in all test files.

window.createRecordingCloseWatcher = (t, events, name) => {
  const prefix = name === undefined ? "" : name + " ";;

  const watcher = new CloseWatcher();
  t.add_cleanup(() => watcher.destroy());
  watcher.addEventListener("cancel", () => events.push(prefix + "cancel"));
  watcher.addEventListener("close", () => events.push(prefix + "close"));

  return watcher;
};

window.createBlessedRecordingCloseWatcher = (t, events, name) => {
  return test_driver.bless("create " + name, () => createRecordingCloseWatcher(t, events, name));
};

window.sendEscKey = () => {
  // *not* \uu001B; see https://w3c.github.io/webdriver/#keyboard-actions
  const ESC = '\uE00C';

  return test_driver.send_keys(document.getElementById("d"), ESC);
};

// For now, we always use the Esc keypress as our close signal. In
// theory, in the future, we could add a WebDriver command or similar
// for the close signal, which would allow different tests on platforms
// with different close signals. In that case, we'd update this
// function, but not update the sendEscKey function above.
window.sendCloseSignal = window.sendEscKey;
