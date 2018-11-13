// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

'use strict';

idl_test(
  ['speech-api'],
  ['dom', 'html'],
  idl_array => {
    idl_array.add_objects({
      SpeechGrammar: ['new SpeechGrammar()'],
      SpeechGrammarList: ['new SpeechGrammarList()'],
      SpeechRecognition: ['new SpeechRecognition()'],
      // TODO: SpeechRecognitionAlternative
      // TODO: SpeechRecognitionErrorEvent
      // TODO: SpeechRecognitionEvent
      // TODO: SpeechRecognitionResult
      // TODO: SpeechRecognitionResultList
      SpeechSynthesis: ['speechSynthesis'],
      // TODO: SpeechSynthesisErrorEvent
      // TODO: SpeechSynthesisEvent
      SpeechSynthesisUtterance: ['new SpeechSynthesisUtterance()'],
      Window: ['self'],
    });

    // https://w3c.github.io/speech-api/#dom-speechsynthesis-getvoices can
    // return an empty list, so add SpeechSynthesisVoice conditionally.
    const voices = speechSynthesis.getVoices();
    if (voices.length) {
      self.voice = voices[0];
      idl_array.add_objects({ SpeechSynthesisVoice: ['voice'] });
    }
  }
);
