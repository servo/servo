const s = new CSSUnparsedValue(['if(supports(animation: a): 1px)']).toString();
postMessage('CSSUnparsedValue.toString() returned ' + JSON.stringify(s));
