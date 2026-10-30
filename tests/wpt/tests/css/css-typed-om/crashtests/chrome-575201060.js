const s = new CSSUnparsedValue([
            'if(supports(list-style-type: disc): 13px)'
          ]).toString();
postMessage('CSSUnparsedValue.toString() returned ' + JSON.stringify(s));
