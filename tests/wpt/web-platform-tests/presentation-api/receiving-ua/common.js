(window => {
  // Both a controlling side and a receiving one must share the same Stash ID to
  // transmit data from one to the other. On the other hand, due to polling mechanism
  // which cleans up a stash, stashes in both controller-to-receiver direction
  // and one for receiver-to-controller are necessary.
  window.stashIds = {
    toController: '0c382524-5738-4df0-837d-4f53ea8addc2',
    toReceiver: 'a9618cd1-ca2b-4155-b7f6-630dce953c44'
  }

  // handle a test result received from a receiving page
  const parseValue = value => {
    let r;

    // String
    if (r = value.match(/^(\(string\)\s+)?"(.*)"$/))
      return r[2];
    // Object
    else if (r = value.match(/^(\(object\)\s+)?object\s+"\[object\s+(.*)\]"$/))
      return window[r[2]].prototype;
    // Number, boolean, null, undefined
    else {
      if (r = value.match(/^(\(\S+\)\s+)?(\S+)$/)) {
        try {
          return JSON.parse(r[2]);
        } catch(e) {
          return value;
        }
      }
      else
        return value;
    }
  };

  window.parseResult = message => {
    let r = message.match(/^(assert_.*):\s+(.*)$/);
    if (r) {
      const assertion = r[1];
      const body = r[2];
      let args;
      switch (assertion) {
        case 'assert_equals':
          if (r = body.match(/^((.*)\s+)?expected\s+((\(\S*\)\s+)?(\S+|(\S+\s+)?\".*\"))\s+but\s+got\s+((\(\S*\)\s+)?(\S+|(\S+\s+)?\".*\"))$/))
            args = [parseValue(r[7]), parseValue(r[3]), r[2]];
          break;
        case 'assert_true':
          if (r = body.match(/^((.*)\s+)?expected\s+(true|false)\s+got\s+(\S+|(\S+\s+)?\".*\")$/))
            args = [parseValue(r[4]), r[2]];
          break;
        case 'assert_unreached':
          if (r = body.match(/^((.*)\s+)?Reached\s+unreachable\s+code$/))
            args = [r[2]];
          break;
      }
      if (args) {
        window[assertion](args[0], args[1], args[2]);
        return;
      }
    }
    // default
    assert_unreached('Test result received from a receiving user agent: ' + message + ': ');
  };
})(window);