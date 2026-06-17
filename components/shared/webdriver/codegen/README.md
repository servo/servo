# codegen for webdriver bidi

## Convertion rule

> [!NOTE]
>
> This part is non-normative examples.

<table>
<thead>
<tr>
<th>Case</th>
<th>CDDL Rule</th>
<th>Rust (serde)</th>
</tr>
</thead>
<tbody>
<tr>
<th>Primitive</th>
<td>

```cddl
text
number
any
0.1..2.0
```

</td>
<td>

```rust
String
f64
serde_json::Value
f64
```

</td>
</tr>
<tr>
<th>Alias</th>
<td>

```cddl
js-int =
  -9007199254740991..
    9007199254740991

js-uint =
  0..9007199254740991

browser.ClientWindow = text;
```

</td>
<td>

```rust
// ignore range, keep type only
type JsInt = i64;
type JsUint = u64;

mod browser {
  type ClientWindow = String;
}
```

TODO: should this be newtype pattern?

</td>
</tr>
<tr>
<th>Plain struct</th>
<td>

```cddl
Command = {
  id: js-uint,
  CommandData,
  Extensible,
}
```

</td>
<td>

```rust
#[derive(Deserialize, Serialize)]
struct Command {
  id: JsUint,
  // inline group are flattened
  #[serde(flatten)]
  command_data: CommandData,
  #[serde(flatten)]
  extensible: Extensible,
}
```

</td>
</tr>
<tr>
<th>Literal enum</th>
<td>

```cddl
ErrorCode =
  "invalid argument" /
  "invalid selector" /
  "invalid session id" /
  ...

script.SpecialNumber =
 "NaN" /
 "-0" /
 "Infinity" /
 "-Infinity";
```

</td>
<td>

```rust
#[derive(Deserialize, Serialize)]
enum ErrorCode {
  #[serde(rename = "invalid argument")]
  InvalidArgument,
  #[serde(rename = "invalid selector")]
  InvalidSelector,
  #[serde(rename = "invalid session id")]
  InvalidSessoinId,
}

#[derive(Deserialize, Serialize)]
mod script {
  enum SpecialNumber {
    NaN,
    #[serde(rename = "-0")]
    NegZero,
    Infinity,
    #[serde(rename = "-Infinity")]
    NegInfinity
  }
}
```

TODO: Single literal should also be enum.

</td>
</tr>
<tr>
<th>Enum Struct</th>
<td>

```cddl
Message = (
  CommandResponse /
  ErrorResponse /
  Event
)

CommandResponse = {
  type: "success",
  id: js-uint,
  result: ResultData,
  Extensible
}
```

</td>
<td>

```rust
#[derive(Deserialize, Serialize)]
enum Message {
  // tag field is moved from sub-struct here
  #[tag = "success"]
  CommandResponse(CommandResponse),
  #[tag = "error"]
  ErrorResponse(ErrorResponse),
  #[tag = "event"]
  Event(Event),
}

#[derive(Deserialize, Serialize)]
pub struct CommandResponse {
  id: JUint,
  result: ResultData,
  #[serde(flatten)]
  extensible: Extensible
}
```

1. sometimes tag field can be `text`, we use `serde(untagged)` to fallback.

</td>
</tr>
<tr>
<th>Optional</th>
<td>

```cddl
ErrorResponse = {
  type: "error",
  id: js-uint / null,
  error: ErrorCode,
  message: text,
  ? stacktrace: text,
  Extensible
}
```

</td>
<td>

```rust
#[derive(Deserialize, Serialize)]
pub struct ErrorResponse {
  id: Option<u64>,
  error: ErrorCode,
  message: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  stacktrace: Option<String>,
  #[serde(flatten)]
  extensible: Extensible
}
```

</td>
</tr>
<tr>
<th>HashMap, Vec, Inline</th>
<td>

```cddl
Extensible = (*text => any)

script.NodeProperties = {
  nodeType: js-uint,
  childNodeCount: js-uint,
  ? attributes: {*text => text},
  ? children: [*script.NodeRemoteValue],
  ? localName: text,
  ? mode: "open" / "closed",
  ? namespaceURI: text,
  ? nodeValue: text,
  ? shadowRoot: script.NodeRemoteValue / null,
}
```

</td>
<td>

```rust
type Extensible = HashMap<String, Value>;

mod script {
  #[derive(Deserialize, Serialize)]
  struct NodeProperties {
    // hashmap here
    #[serde(skip_serializing_if = "Option::is_none")]
    attributes: Option<HashMap<String, String>>,
    // vec here
    #[serde(skip_serializing_if = "Option::is_none")]
    children: Option<Vec<NodeRemoteValue>>,
    // inline here
    #[serde(skip_serializing_if = "Option::is_none")]
    mode: Option<NodePropertiesMode>,
    // both optional and skip
    #[serde(skip_serializing_if = "Option::is_none")]
    shadow_root: Option<NodeRemoteValue>,
    ..
  }

  // derived enum is put in same module
  #[derive(Deserialize, Serialize)]
  enum NodePropertiesMode {
    #[serde(rename = "open")]
    Open,
    #[serde(rename = "closed")]
    Closed,
  }
}

```

</td>
</tr>
<tr>
<th>Nested enum</th>
<td>

```cddl
CommandData = (
  BrowserCommand //
  BrowsingContextCommand //
  EmulationCommand //
  InputCommand //
  NetworkCommand //
  ScriptCommand //
  SessionCommand //
  StorageCommand //
  WebExtensionCommand
)
```

</td>
<td>

```rust
#[derive(Deserialize, Serialize)]
enum CommandData {
  #[serde(untagged)]
  BrowserCommand(BrowserCommand),
  #[serde(untagged)]
  BrowsingContextCommand(BrowsingContextCommand),
  ..
}
```

1. Some enum may be mixed nested and struct enum,
   serde allow us to mix tag and untagged
   (e.g. `script.RemoteValue`)

</td>
</tr>
</tbody>
</table>

## Edge cases

- [ ] degenerate single field struct, like `NullValue`
- [ ] fix `DownloadBehavior` type not recognized as enum
- [ ] impl From for groups, e.g. `From<InputResult> for ResultData`
- [ ] fix `LogEvent::Added`, method
- [ ] script.NumberValueValue, the name is cumbersome, and seems that untagged is not used
- [ ] fix `script.WindowRealmInfo` type not recognized
