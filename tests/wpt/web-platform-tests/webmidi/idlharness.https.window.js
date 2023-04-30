// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js
// META: timeout=long

// https://webaudio.github.io/web-midi-api/

'use strict';

idl_test(
  ['webmidi'],
  ['html', 'dom', 'permissions'],
  async idl_array => {
    idl_array.add_objects({
      MIDIPort: [],
      MIDIMessageEvent: [
        'new MIDIMessageEvent("type", { data: new Uint8Array([0]) })'
      ],
      MIDIConnectionEvent: ['new MIDIConnectionEvent("type")'],
    })

    self.access = await navigator.requestMIDIAccess();
    self.inputs = access.inputs;
    self.outputs = access.outputs;
    idl_array.add_objects({ MIDIInputMap: ['inputs'] });
    idl_array.add_objects({ MIDIOutputMap: ['outputs'] });
    idl_array.add_objects({ MIDIAccess: ['access'] });
    if (inputs.size) {
      self.input = Array.from(access.inputs.values())[0];
      idl_array.add_objects({ MIDIInput: ['input'] });
    }
    if (outputs.size) {
      self.output = Array.from(access.outputs.values())[0];
      idl_array.add_objects({ MIDIOutput: ['output'] });
    }
  }
);
