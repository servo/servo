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
  // Esc is \uE00C, *not* \uu001B; see https://w3c.github.io/webdriver/#keyboard-actions.
  //
  // It's important to target document.body, and not any element that might stop receiving events
  // if a popover or dialog is making that element inert.
  return test_driver.send_keys(document.body, '\uE00C');
};

// For now, we always use the Esc keypress as our close request. In
// theory, in the future, we could add a WebDriver command or similar
// for the close request, which would allow different tests on platforms
// with different close requests. In that case, we'd update this
// function, but not update the sendEscKey function above.
window.sendCloseRequest = window.sendEscKey;
