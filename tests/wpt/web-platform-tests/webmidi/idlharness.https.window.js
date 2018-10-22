// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

// https://webaudio.github.io/web-midi-api/

'use strict';

idl_test(
  ['webmidi'],
  ['html', 'dom'],
  async idl_array => {
    idl_array.add_objects({
      MIDIAccess: ['access'],
      MIDIInputMap: ['inputs'],
      MIDIOutputMap: ['outputs'],
      MIDIConnectionEvent: ['new MIDIConnectionEvent("type")'],
    })

    self.access = await navigator.requestMIDIAccess();
    self.inputs = access.inputs;
    if (inputs.size) {
      self.input = Array.from(access.inputs.values())[0];
      idl_array.add_objects({ MIDIInput: ['input'] });
    }
    self.outputs = access.outputs;
    if (outputs.size) {
      self.output = Array.from(access.outputs.values())[0];
      idl_array.add_objects({ MIDIOutput: ['output'] });
    }
  }
);
