// META: script=/resources/WebIDLParser.js
// META: script=/resources/idlharness.js

let idlArray,
    idl = `[Constructor(USVString url, optional USVString base),
 Exposed=(Window,Worker),
 LegacyWindowAlias=webkitURL]
interface URL {
  stringifier attribute USVString href;
  readonly attribute USVString origin;
           attribute USVString protocol;
           attribute USVString username;
           attribute USVString password;
           attribute USVString host;
           attribute USVString hostname;
           attribute USVString port;
           attribute USVString pathname;
           attribute USVString search;
  [SameObject] readonly attribute URLSearchParams searchParams;
           attribute USVString hash;

  USVString toJSON();
};

[Constructor(optional (sequence<sequence<USVString>> or record<USVString, USVString> or USVString) init = ""),
 Exposed=(Window,Worker)]
interface URLSearchParams {
  void append(USVString name, USVString value);
  void delete(USVString name);
  USVString? get(USVString name);
  sequence<USVString> getAll(USVString name);
  boolean has(USVString name);
  void set(USVString name, USVString value);

  void sort();

  iterable<USVString, USVString>;
  stringifier;
};`;
setup(function() {
  idlArray = new IdlArray();
  idlArray.add_idls(idl);
}, {explicit_done:true});

idlArray.add_objects({
  URL: ['new URL("http://foo")'],
  URLSearchParams: ['new URLSearchParams("hi=there&thank=you")']
});
idlArray.test();

done();
