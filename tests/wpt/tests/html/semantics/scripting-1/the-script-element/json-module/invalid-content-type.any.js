// META: global=window,dedicatedworker,sharedworker

const content_types = [
  "application/json+protobuf",
  "application/json+blah",
  "text/x-json",
  "text/json+blah",
  "application/blahjson",
  "image/json",
  "text+json",
  "json+json",
  "text/json/json+json",
  "text/html;+json",
  "text/html+json+xml",
  "text/json/json",

  // Control Characters and Whitespace - Invalid Type
  "applic\x00ation/vnd.api+json",     // NULL in type
  "applic\x09ation/vnd.api+json",     // TAB in type
  "applic\x0Aation/vnd.api+json",     // Line Feed in type
  "applic\x0Dation/vnd.api+json",     // Carriage Return in type
  "applic ation/vnd.api+json",        // SPACE in type
  "applic\x7Fation/vnd.api+json",     // DEL in type
  "\x01application/vnd.api+json",     // SOH at start of type
  "application\x1F/vnd.api+json",     // Unit Separator at end of type

  // Control Characters and Whitespace - Invalid Subtype
  "application/vnd\x00.api+json",     // NULL in subtype
  "application/vnd\x09.api+json",     // TAB in subtype
  "application/vnd\x0A.api+json",     // Line Feed in subtype
  "application/vnd\x0D.api+json",     // Carriage Return in subtype
  "application/vnd .api+json",        // SPACE in subtype
  "application/vnd\x7F.api+json",     // DEL in subtype
  "application/\x01vnd.api+json",     // SOH at start of subtype
  "application/vnd.api\x1F+json",     // Unit Separator before +json

  // Separator Characters - Invalid Type
  "applic\"ation/vnd.api+json",       // Double quote in type
  "applic(ation/vnd.api+json",        // Left parenthesis in type
  "applic)ation/vnd.api+json",        // Right parenthesis in type
  "applic,ation/vnd.api+json",        // Comma in type
  "applic:ation/vnd.api+json",        // Colon in type
  "applic;ation/vnd.api+json",        // Semicolon in type
  "applic<ation/vnd.api+json",        // Left angle bracket in type
  "applic>ation/vnd.api+json",        // Right angle bracket in type
  "applic=ation/vnd.api+json",        // Equals in type
  "applic?ation/vnd.api+json",        // Question mark in type
  "applic@ation/vnd.api+json",        // At sign in type
  "applic[ation/vnd.api+json",        // Left square bracket in type
  "applic]ation/vnd.api+json",        // Right square bracket in type
  "applic{ation/vnd.api+json",        // Left curly brace in type
  "applic}ation/vnd.api+json",        // Right curly brace in type

  // Separator Characters - Invalid Subtype
  "application/vnd\"api\"+json",      // Double quote in subtype
  "application/vnd(api+json",         // Left parenthesis in subtype
  "application/vnd)api+json",         // Right parenthesis in subtype
  "application/vnd,api+json",         // Comma in subtype
  "application/vnd:api+json",         // Colon in subtype
  "application/vnd;api+json",         // Semicolon in subtype
  "application/vnd<api+json",         // Left angle brackets in subtype
  "application/vnd>api+json",         // Right angle brackets in subtype
  "application/vnd=api+json",         // Equals in subtype
  "application/vnd?api+json",         // Question mark in subtype
  "application/vnd@api+json",         // At sign in subtype
  "application/vnd[api+json",         // Left square brackets in subtype
  "application/vnd]api+json",         // Right square brackets in subtype
  "application/vnd{api+json",         // Left curly brace in subtype
  "application/vnd}api+json",         // Right curly brace in subtype

  // Non-ASCII Characters - Invalid Type
  "aplicación/vnd.api+json",          // Latin small letter o with acute
  "申请/vnd.api+json",                 // Chinese characters
  "app™lication/vnd.api+json",        // Trade mark sign
  "appli€cation/vnd.api+json",        // Euro sign
  "🚀application/vnd.api+json",       // Rocket emoji
  "applicatioñ/vnd.api+json",         // Latin small letter n with tilde

  // Non-ASCII Characters - Invalid Subtype
  "application/vñd.api+json",         // Latin small letter n with tilde
  "application/vnd.apí+json",         // Latin small letter i with acute
  "application/vnd.api™+json",        // Trade mark sign
  "application/vnd.api€+json",        // Euro sign
  "application/vnd.中文+json",         // Chinese characters
  "application/vnd.api🚀+json",       // Rocket emoji
  "application/café.api+json",        // Latin small letter e with acute

  // Mixed Invalid Characters (Both Type and Subtype)
  "applic ation/vnd api+json",        // Spaces in both
  "applic\"ation/vnd\"api+json",      // Quotes in both
  "applic(ation/vnd(api+json",        // Left parentheses in both
  "applic)ation/vnd)api+json",        // Right parentheses in both
  "applic,ation/vnd,api+json",        // Commas in both
  "applic=ation/vnd=api+json",        // Equals in both
  "申请/中文.api+json",                 // Chinese in both
  "app™/vnd€.api+json",               // Unicode symbols in both
  "applic\x00ation/vnd\x00api+json",  // NULL in both
  "applic;ation/vnd;api+json",        // Semicolons in both
  "applic{ation/vnd{api+json",        // Left curly brace in both
  "applic}ation/vnd}api+json",        // Right curly brace in both
  "applic[ation/vnd[api+json",        // Left square bracket in both
  "applic]ation/vnd]api+json",        // Right square bracket in both
  "applic<ation/vnd<api+json",        // Left angle bracket in both
  "applic>ation/vnd>api+json",        // Right angle bracket in both

  // Edge Cases - Type
  "\"application/vnd.api+json",       // Quote at start of type
  "application\"/vnd.api+json",       // Quote at end of type
  "application /vnd.api+json",        // Trailing space in type
  "/vnd.api+json",                    // Empty type
  "app\x00lication/vnd.api+json",     // NULL in middle of type

  // Edge Cases - Subtype
  "application/\"vnd.api+json",       // Quote at start of subtype
  "application/vnd.api\"+json",       // Quote before +json
  "application/ vnd.api+json",        // Leading space in subtype
  "application/vnd.api +json",        // Space before +json
  "application/vnd.api+json\"",       // Quote at end

  // Edge Cases - Multiple Invalid Positions
  "\"application\"/\"vnd.api\"+json", // Quotes in multiple positions
  "app(lic)ation/vnd(api)+json",      // Parentheses in multiple positions
  "application\x00/\x00vnd.api+json",  // NULL in both parts
];

for (const content_type of content_types) {
  promise_test(async test => {
    await promise_rejects_js(test, TypeError,
      import(`./module.json?pipe=header(Content-Type,${encodeURIComponent(content_type)})`, { with: { type: "json"} }),
      `Import of a JSON module with MIME type ${content_type} should fail`);
  }, `Try importing JSON module with MIME type ${content_type}`);
}

// { type: "json" } should not load content with JavaScript MIME type as JavaScript.
const js_content_types = [
  "application/ecmascript",
  "application/javascript",
  "application/x-ecmascript",
  "application/x-javascript",
  "text/ecmascript",
  "text/javascript",
]

for (const content_type of js_content_types) {
  promise_test(async test => {
    await promise_rejects_js(test, TypeError,
      import(`./should-not-be-loaded.js?pipe=header(Content-Type,${encodeURIComponent(content_type)})`, { with: { type: "json"} }),
      `Importing JSON module should not load it as JavaScript when its MIME type is ${content_type}`);
  }, `Try importing JSON module as JavaScript with MIME type ${content_type}`);
}
