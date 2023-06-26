async function runOpaqueAdSizesTest(input_width, input_height, output_width, output_height) {
  // Attach a FLEDGE fenced frame whose outer container has dimensions
  // `input_width` by `input_height`.
  const frame = await attachFencedFrameContext({
      generator_api: "fledge", resolve_to_config: true, attributes: [
      ["width", input_width], ["height", input_height]]});

  const assert_dimensions =
      (label, input_width, input_height, output_width, output_height) => {
    assert_equals(getComputedStyle(document.documentElement).width,
        output_width+"px",
        label + " the computed width coerces to " + output_width);
    assert_equals(window.innerWidth, output_width,
        label + " the innerWidth " + input_width + " coerces to " + output_width);
    assert_equals(window.innerHeight, output_height,
        label + " the innerHeight " + input_height + " coerces to " + output_height);
  }

  // Assert that the fenced frame sees its dimensions rounded to the nearest
  // ad size.
  await frame.execute(assert_dimensions,
      ["After navigation", input_width, input_height, output_width, output_height]);

  // Assert that the embedder sees the fenced frame's original dimensions.
  assert_equals(frame.width, input_width.toString(),
      "The outer container width is the requested width.");
  assert_equals(frame.height, input_height.toString(),
      "The outer container height is the requested height.");

  // Resize the fenced frame's outer container.
  const new_size_x = 320;
  const new_size_y = 50;
  frame.width = new_size_x;
  frame.height = new_size_y;

  // Refresh the fenced frame.
  await frame.execute(() => {
    window.executor.suspend(() => {
      location.href = location.href;
    });
  });

  // Observe that navigations after the first don't change the fenced frame's
  // inner dimensions.
  await frame.execute(assert_dimensions,
      ["After resizing", input_width, input_height, output_width, output_height]);
}
