# WebDriver specification tests

Herein lies a set of conformance tests
for the W3C web browser automation specification
known as [WebDriver](http://w3c.github.io/webdriver/webdriver-spec.html).
The purpose of these tests is determine implementation compliance
so that different driver implementations can determine
whether they meet the recognised standard.

## Chapters of the Spec that still need tests

Note: Sections that are currently we believe are not quite stable enough for tests yet are in <span style="color:red;">red</span>.
Note: Sections that likely have enough tests for now are marked in <span style="color:green;">green</span>.

* Routing Requests
* List of Endpoints (existance tests)
* List of Error Codes (Description is NON Normative)
* Capabilities
* Sessions
* Delete Session
* Set Timeouts
* Navigation
** Get Current URL
** Back
** Forward
** Refresh
** Get Title
* Command Contexts
** Get Window Handle
** Close Window
** Switch To Window
** Get Window Handles
** Switch To Frame
** Switch To Parent Frame
* Resizing and Positioning Windows
** Get Window Size
** Set Window Size
** Get Window Position
** Set Window Position
** Maximize Window
** Minimize Window
** Fullscreen Window
* Elements
** Element Interactability
** Get Active Element
* Element Retrieval
** Locator Strategies
*** CSS Selectors
*** Link Text
*** Partial Link Text
*** XPath
** Find Element
** Find Elements
** Find Element from Element
** Find Elements from Element
* Element State
** Is Element Selected
** Get Element Attribute
** Get Element Property
** Get Element CSS value
** Get Element Text
** Get Element Tag name
** Get Element Rect
** Is Element Enabled
* Element Interaction
** Element Click
** Element Clear
** Element Send Keys
* Document Handling
** Getting Page Source
** Executing Script
** Execute Script
** Execute Async Script
* Cookies
** Get All Cookies
** Get Named Cookies
** Add Cookie
** Delete Cookie
** Delete All Cookies
* <span style="color:red;">Actions
** Input State
** Processing Actions Requests
** Dispatching Actions
** General Actions
** Keyboard Actions
** Pointer Actions
** Perform Actions
** Remote End Steps (non-Normative)
** Releasing Actions</span>
* User Prompts
** Dismiss Alert
** Accept Alert
** Get Alert Text
** Send Alert Text
* Screen Capture
** Take Screenshot
** Take Element Screenshot
* <span style="color:green;">Privacy</span>
* <span style="color:green;">Security</span>
* Element Displayedness