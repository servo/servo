async function pinch_zoom_action(targetWindow = window) {
  // Pinch zoom in this document.
  await new test_driver.Actions()
    .addPointer("finger1", "touch")
    .addPointer("finger2", "touch")
    .pointerMove(parseInt(targetWindow.innerWidth / 2),
                 parseInt(targetWindow.innerHeight / 2),
                 {origin: "viewport", sourceName: "finger1"})
    .pointerMove(parseInt(targetWindow.innerWidth / 2),
                 parseInt(targetWindow.innerHeight / 2),
                 {origin: "viewport", sourceName: "finger2"})
    .pointerDown({sourceName: "finger1"})
    .pointerDown({sourceName: "finger2"})
    .pointerMove(parseInt(targetWindow.innerWidth / 3),
                 parseInt(targetWindow.innerHeight / 3),
                 {origin: "viewport", sourceName: "finger1"})
    .pointerMove(parseInt(targetWindow.innerWidth / 3 * 2),
                 parseInt(targetWindow.innerHeight / 3 * 2),
                 {origin: "viewport", sourceName: "finger2"})
    .pointerUp({sourceName: "finger1"})
    .pointerUp({sourceName: "finger2"})
    .send();
}
