// XPCShell script for generating a single file containing all certificates in PEM
// format. You may run this in the browser toolbox's console
// (Firefox -> devtools -> settings -> enable remote/chrome debugging,
// followed by settings -> devtools menu -> browser toolbox) or the 
// xpcshell runner that comes with a built Firefox (./run-mozilla.sh ./xpcshell).
// The variable `certstring` contains the final pem file. You can use `save(path)` to
// save it to a file. `certlist` contains an array with the PEM certs as well as their names if you
// want to filter them.


// http://mxr.mozilla.org/mozilla-central/source/security/manager/pki/resources/content/pippki.js
function getDERString(cert)
{
  var length = {};
  var derArray = cert.getRawDER(length);
  var derString = '';
  for (var i = 0; i < derArray.length; i++) {
    derString += String.fromCharCode(derArray[i]);
  }
  return derString;
}

// http://mxr.mozilla.org/mozilla-central/source/security/manager/pki/resources/content/pippki.js
function getPEMString(cert)
{
  var derb64 = btoa(getDERString(cert));
  // Wrap the Base64 string into lines of 64 characters, 
  // with CRLF line breaks (as specified in RFC 1421).
  var wrapped = derb64.replace(/(\S{64}(?!$))/g, "$1\r\n");
  return "-----BEGIN CERTIFICATE-----\r\n"
         + wrapped
         + "\r\n-----END CERTIFICATE-----\r\n";
}

let certdb = Components.classes["@mozilla.org/security/x509certdb;1"].createInstance(Ci.nsIX509CertDB);
let enumerator = certdb.getCerts().getEnumerator();
let certlist = [];
let certstring="";
while(enumerator.hasMoreElements()){
  let cert = enumerator.getNext().QueryInterface(Ci.nsIX509Cert);
  let pem = getPEMString(cert);
  let trusted = certdb.isCertTrusted(cert, Ci.nsIX509Cert.CA_CERT, Ci.nsIX509CertDB.TRUSTED_SSL);
  certlist.push({name: cert.commonName, pem: pem, trusted: trusted});
  if (trusted) {
    certstring+=pem;
  }
}

function save(path) {
  // https://developer.mozilla.org/en-US/Add-ons/Code_snippets/File_I_O
  Components.utils.import("resource://gre/modules/FileUtils.jsm");
  var file = new FileUtils.File(path);
  Components.utils.import("resource://gre/modules/NetUtil.jsm");

  // file is nsIFile, data is a string

  // You can also optionally pass a flags parameter here. It defaults to
  // FileUtils.MODE_WRONLY | FileUtils.MODE_CREATE | FileUtils.MODE_TRUNCATE;
  var ostream = FileUtils.openSafeFileOutputStream(file);

  var converter = Components.classes["@mozilla.org/intl/scriptableunicodeconverter"].
                  createInstance(Components.interfaces.nsIScriptableUnicodeConverter);
  converter.charset = "UTF-8";
  var istream = converter.convertToInputStream(certstring);

  // The last argument (the callback) is optional.
  NetUtil.asyncCopy(istream, ostream, function(status) {
    if (!Components.isSuccessCode(status)) {
      // Handle error!
      return;
    }

    // Data has been written to the file.
  });
}
