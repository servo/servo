const testcases = [
  {config_input: {}, value: "test", result: "test", message: "string"},
  {config_input: {}, value: "<b>bla</b>", result: "<b>bla</b>", message: "html fragment"},
  {config_input: {}, value: "<a<embla", result: "", message: "broken html"},
  {config_input: {}, value: {}, result: "[object Object]", message: "empty object"},
  {config_input: {}, value: 1, result: "1", message: "number"},
  {config_input: {}, value: 000, result: "0", message: "zeros"},
  {config_input: {}, value: 1+2, result: "3", message: "arithmetic"},
  {config_input: {}, value: "", result: "", message: "empty string"},
  {config_input: {}, value: "<html><head></head><body>test</body></html>", result: "test", message: "document"},
  {config_input: {}, value: "<div>test", result: "<div>test</div>", message: "html without close tag"},
];
