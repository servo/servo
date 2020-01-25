// META: title=XMLHttpRequest.send(ES object)

function do_test(obj, expected, name) {
  var test = async_test(name)
  test.step(function() {
    var client = new XMLHttpRequest()
    client.onload = test.step_func(function () {
      assert_equals(client.responseText, expected)
      test.done()
    });
    client.open('POST', 'resources/content.py')
    if (expected.exception) {
      if (expected.exception.identity) {
        assert_throws_exactly(expected.exception.identity,
                              function(){client.send(obj)})
      } else {
        assert_throws_js(expected.exception.ctor,
                         function(){client.send(obj)})
      }
      test.done()
    } else {
      client.send(obj)
    }
  });
}

do_test({}, '[object Object]', 'sending a plain empty object')
do_test(Math, '[object Math]', 'sending the ES Math object')
do_test(new XMLHttpRequest, '[object XMLHttpRequest]', 'sending a new XHR instance')
do_test({toString:function(){}}, 'undefined', 'sending object that stringifies to undefined')
do_test({toString:function(){return null}}, 'null', 'sending object that stringifies to null')
var ancestor = {toString: function(){
  var ar=[]
  for (var prop in this) {
    if (this.hasOwnProperty(prop)) {
      ar.push(prop+'='+this[prop])
    }
  };
  return ar.join('&')
}};

var myObj = Object.create(ancestor, {foo:{value:1, enumerable: true},  bar:{value:'foo', enumerable:true}})
do_test(myObj, 'foo=1&bar=foo', 'object that stringifies to query string')

var myFakeJSON = {a:'a', b:'b', toString:function(){ return JSON.stringify(this, function(key, val){ return key ==='toString'?undefined:val; }) }}
do_test(myFakeJSON, '{"a":"a","b":"b"}', 'object that stringifies to JSON string')

var myFakeDoc1 = {valueOf:function(){return document}}
do_test(myFakeDoc1, '[object Object]', 'object whose valueOf() returns a document - ignore valueOf(), stringify')

var myFakeDoc2 = {toString:function(){return document}}
var expectedError = self.GLOBAL.isWorker() ? ReferenceError : TypeError;
do_test(myFakeDoc2, {exception: { ctor: expectedError } }, 'object whose toString() returns a document, expected to throw')

var err = {name:'FooError', message:'bar'};
var myThrower = {toString:function(){throw err;}};
do_test(myThrower, {exception: { identity: err }}, 'object whose toString() throws, expected to throw')
