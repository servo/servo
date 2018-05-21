// Test k-rate vs a-rate AudioParams.
//
// |options| describes how the testing of the AudioParam should be done:
//
//   nodeName:  name of the AudioNode to be tested
//   nodeOptions:  options to be used in the AudioNode constructor
//
//   prefix: Prefix for all output messages (to make them unique for
//           testharness)
//
//   rateSettings: A vector of dictionaries specifying how to set the automation
//                 rate(s):
//       name: Name of the AudioParam
//       value: The automation rate for the AudioParam given by |name|.
//
//   automations: A vector of dictionaries specifying how to automate each
//                AudioParam:
//       name: Name of the AudioParam
//
//       methods: A vector of dictionaries specifying the automation methods to
//                be used for testing:
//           name: Automation method to call
//           options: Arguments for the automation method
//
// Testing is somewhat rudimentary.  We create two nodes of the same type.  One
// node uses the default automation rates for each AudioParam (expecting them to
// be a-rate).  The second node sets the automation rate of AudioParams to
// "k-rate".  The set is speciified by |options.rateSettings|.
//
// For both of these nodes, the same set of automation methods (given by
// |options.automations|) is applied.  A simple oscillator is connected to each
// node which in turn are connected to different channels of an offline context.
// Channel 0 is the k-rate node output; channel 1, the a-rate output; and
// channel 3, the difference between the outputs.
//
// Success is declared if the difference signal is not exactly zero.  This means
// the the automations did different things, as expected.
//
// The promise from |startRendering| is returned.
function doTest(context, should, options) {
  let merger = new ChannelMergerNode(
      context, {numberOfInputs: context.destination.numberOfChannels});
  merger.connect(context.destination);

  let src = new OscillatorNode(context);
  let kRateNode = new window[options.nodeName](context, options.nodeOptions);
  let aRateNode = new window[options.nodeName](context, options.nodeOptions);
  let inverter = new GainNode(context, {gain: -1});

  // Set kRateNode filter to use k-rate params.
  options.rateSettings.forEach(setting => {
    kRateNode[setting.name].automationRate = setting.value;
    // Mostly for documentation in the output.  These should always
    // pass.
    should(
        kRateNode[setting.name].automationRate,
        `${options.prefix}: Setting ${
                                      setting.name
                                    }.automationRate to "${setting.value}"`)
        .beEqualTo(setting.value);
  });

  // Run through all automations for each node separately. (Mostly to keep
  // output of automations together.)
  options.automations.forEach(param => {
    param.methods.forEach(method => {
      // Most for documentation in the output.  These should never throw.
      let message = `${param.name}.${method.name}(${method.options})`
      should(() => {
        kRateNode[param.name][method.name](...method.options);
      }, options.prefix + ': k-rate node: ' + message).notThrow();
    });
  });
  options.automations.forEach(param => {
    param.methods.forEach(method => {
      // Most for documentation in the output.  These should never throw.
      let message = `${param.name}.${method.name}(${method.options})`
      should(() => {
        aRateNode[param.name][method.name](...method.options);
      }, options.prefix + ': a-rate node:' + message).notThrow();
    });
  });

  // The k-rate result is channel 0, and the a-rate result is channel 1.
  src.connect(kRateNode).connect(merger, 0, 0);
  src.connect(aRateNode).connect(merger, 0, 1);

  // Compute the difference between the a-rate and k-rate results and send
  // that to channel 2.
  kRateNode.connect(merger, 0, 2);
  aRateNode.connect(inverter).connect(merger, 0, 2);

  src.start();
  return context.startRendering().then(renderedBuffer => {
    let kRateOutput = renderedBuffer.getChannelData(0);
    let aRateOutput = renderedBuffer.getChannelData(1);
    let diff = renderedBuffer.getChannelData(2);

    // Some informative messages to print out values of the k-rate and
    // a-rate outputs.  These should always pass.
    should(
        kRateOutput, `${options.prefix}: Output of k-rate ${options.nodeName}`)
        .beEqualToArray(kRateOutput);
    should(
        aRateOutput, `${options.prefix}: Output of a-rate ${options.nodeName}`)
        .beEqualToArray(aRateOutput);

    // The real test.  If k-rate AudioParam is working correctly, the
    // k-rate result MUST differ from the a-rate result.
    should(
        diff,
        `${
           options.prefix
         }: Difference between a-rate and k-rate ${options.nodeName}`)
        .notBeConstantValueOf(0);
  });
}
