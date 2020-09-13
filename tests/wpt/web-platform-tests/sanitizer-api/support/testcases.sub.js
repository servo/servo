const testcases = [
  {value: "test", result: "test", message: "string"},
  {value: "<b>bla</b>", result: "<b>bla</b>", message: "html fragment"},
  {value: "<a<embla", result: "", message: "broken html"},
  {value: {}, result: "[object Object]", message: "empty object"},
  {value: 1, result: "1", message: "number"},
  {value: 000, result: "0", message: "zeros"},
  {value: 1+2, result: "3", message: "arithmetic"},
  {value: "", result: "", message: "empty string"},
  {value: undefined, result: "undefined", message: "undefined"},
  {value: null, result: "null", message: "null"},
  {value: "<html><head></head><body>test</body></html>", result: "test", message: "document"},
  {value: "<div>test", result: "<div>test</div>", message: "html without close tag"},
  {value: "<script>alert('i am a test')<\/script>", result: "", message: "scripts"},
  {value: "<p onclick='a= 123'>Click.</p>", result: "<p>Click.</p>", message: "onclick scripts"}

];
