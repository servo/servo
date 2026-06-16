# Implementation status of WebDriver

## WebDriver Classic (HTTP)

TODO: write doc for this part.

## WebDriver BiDi

### Overall

| Feature                                 | Status | Details                                     |
| --------------------------------------- | ------ | ------------------------------------------- |
| CDDL to serde codegen                   | ✅     | Some edge cases not handled.                |
| Custom module (`{app}:{module}.{name}`) | ❌     | Our parser cannot handle this currently.    |
| TLS secure connection (`wss://`)        | ❌     |                                             |
| Channel-based connection from embedder  | ❌     | Not in spec, but useful for embedded usage. |

### The `session` module

| Command               | Status | Details                     |
| --------------------- | ------ | --------------------------- |
| `sesion.status`       | ✅     |                             |
| `sesion.new`          | ✅     | Capabilities not processed. |
| `sesion.end`          | ✅     |                             |
| `session.unsubscribe` | ✅     |                             |
| `session.subscribe`   | 🚧     | Trigger not called.         |

### The `browser` module

| Command                        | Status | Details                                  |
| ------------------------------ | ------ | ---------------------------------------- |
| `browser.close`                | ✅     |                                          |
| `browser.getClientWindows`     | 🚧     | Server side done, message not handled.   |
| `browser.setClientWindowState` | ❌     |                                          |
| `browser.createUserContext`    | ⛔     | Blocked by user context not implemented. |
| `browser.getUserContexts`      | ⛔     | Blocked by user context not implemented. |
| `browser.removeUserContext`    | ⛔     | Blocked by user context not implemented. |
| `browser.setDownloadBehavior`  | ⛔     | Blocked by download not implemented.     |

### The `browsingContext` module

| Command                             | Status | Details                                |
| ----------------------------------- | ------ | -------------------------------------- |
| `browsingContext.activate`          | 🚧     | Server side done, message not handled. |
| `browsingContext.captureScreenshot` | 🚧     | Server side done, message not handled. |
| `browsingContext.close`             | 🚧     | Server side done, message not handled. |
| `browsingContext.create`            | 🚧     | Server side done, message not handled. |
| `browsingContext.getTree`           | 🚧     | Server side done, message not handled. |
| `browsingContext.reload`            | 🚧     | Server side done, message not handled. |
| `browsingContext.traverseHistory`   | 🚧     | Server side done, message not handled. |
| `browsingContext.navigate`          | 🚧     | Server side done, message not handled. |
| `browsingContext.locateNodes`       | ❌     |                                        |
| `browsingContext.handleUserPrompt`  | ❌     |                                        |
| `browsingContext.startScreencast`   | ⏳     | Unstable.                              |
| `browsingContext.stopScreencast`    | ⏳     | Unstable.                              |
| `browsingContext.setBypassCSP`      | ⛔     | Blocked by no CSP option.              |
| `browsingContext.setViewport`       | ⛔     | Blocked by viewport not configurable.  |
| `browsingContext.print`             | ⛔     | Blocked by PDF not implemented.        |

| Event                                 | Status | Details                              |
| ------------------------------------- | ------ | ------------------------------------ |
| `browsingContext.contextCreated`      | ❌     |                                      |
| `browsingContext.contextDestroyed`    | ❌     |                                      |
| `browsingContext.navigationStarted`   | ❌     |                                      |
| `browsingContext.fragmentNavigated`   | ❌     |                                      |
| `browsingContext.historyUpdated`      | ❌     |                                      |
| `browsingContext.domContentLoaded`    | ❌     |                                      |
| `browsingContext.load`                | ❌     |                                      |
| `browsingContext.navigationAborted`   | ❌     |                                      |
| `browsingContext.navigationCommitted` | ❌     |                                      |
| `browsingContext.navigationFailed`    | ❌     |                                      |
| `browsingContext.userPromptClosed`    | ❌     |                                      |
| `browsingContext.userPromptOpened`    | ❌     |                                      |
| `browsingContext.downloadWillBegin`   | ⛔     | Blocked by download not implemented. |
| `browsingContext.downloadEnd`         | ⛔     | Blocked by download not implemented. |

### The `emulation` module

| Command                                     | Status  | Details                                                |
| ------------------------------------------- | ------- | ------------------------------------------------------ |
| `emulation.setScreenSettingsOVerride`       | ⛔ easy | Blocked by no screen setting override.                 |
| `emulation.setUserAgentOverride`            | ⛔ easy | Blocked by user agent is global.                       |
| `emulation.setForcedColorsModeThemeOveride` | ⛔      | Blocked by forced colors mode not implemented.         |
| `emulation.setGeolocationOverride`          | ⛔      | Blocked by Geolocation API not implemented.            |
| `emulation.setLocaleOverride`               | ⛔      | Blocked by locale is `OnceLock`.                       |
| `emulation.setNetworkConditions`            | ⛔      | Blocked by offline mode not implemented.               |
| `emulation.setScreenOrientationOverride`    | ⛔      | Blocked by ScreenOrientation API not implemented.      |
| `emulation.setScriptingEnabled`             | ⛔      | Blocked by no such disable scripting option.           |
| `emulation.setScrollbarTypeOverride`        | ⛔      | Blocked by scrollbar type not implemented.             |
| `emulation.setTimezoneOverride`             | ⛔      | Blocked by `ResetRealmTimezone` not exported in mozjs. |
| `emulation.setTouchOverride`                | ⛔      | Blocked by `maxTouch` not implemented.                 |

### The `network` module

| Command                       | Status | Details |
| ----------------------------- | ------ | ------- |
| `network.addDataCollector`    | ❌     |         |
| `network.addIntercept`        | ❌     |         |
| `network.continueRequest`     | ❌     |         |
| `network.continueResponse`    | ❌     |         |
| `network.continueWithAuth`    | ❌     |         |
| `network.disownData`          | ❌     |         |
| `network.failRequest`         | ❌     |         |
| `network.getData`             | ❌     |         |
| `network.provideResponse`     | ❌     |         |
| `network.removeDataCollector` | ❌     |         |
| `network.removeIntercept`     | ❌     |         |
| `network.setCacheBehavior`    | ❌     |         |
| `network.setExtraHeaders`     | ❌     |         |

| Events                      | Status | Details |
| --------------------------- | ------ | ------- |
| `network.authRequired`      | ❌     |         |
| `network.beforeRequestSent` | ❌     |         |
| `network.fetchError`        | ❌     |         |
| `network.responseCompleted` | ❌     |         |
| `network.responseStarted`   | ❌     |         |

### The `script` module

| Commands                     | Status | Details |
| ---------------------------- | ------ | ------- |
| `script.addPreloadScript`    | ❌     |         |
| `script.disown`              | ❌     |         |
| `script.callFunction`        | ❌     |         |
| `script.evaluate`            | ❌     |         |
| `script.getRealms`           | ❌     |         |
| `script.removePreloadScript` | ❌     |         |

| Events                  | Status | Details |
| ----------------------- | ------ | ------- |
| `script.message`        | ❌     |         |
| `script.realmCreated`   | ❌     |         |
| `script.realmDestroyed` | ❌     |         |

### The `storage` module

| Commands                | Status | Details                         |
| ----------------------- | ------ | ------------------------------- |
| `storage.getCookies`    | 🚧     | Channel done, message not sent. |
| `storage.setCookie`     | 🚧     | Channel done, message not sent. |
| `storage.deleteCookies` | 🚧     | Channel done, message not sent. |

### The `log` module

| Events                          | Status | Details                                        |
| ------------------------------- | ------ | ---------------------------------------------- |
| `log.entryAdded` (console part) | ✅     |                                                |
| `log.entryAdded` (jserror part) | ❌     | Do not know where to set global error handler. |

### The `input` module

| Commands               | Status | Details |
| ---------------------- | ------ | ------- |
| `input.performActions` |        |         |
| `input.releaseActions` |        |         |
| `input.setFiles`       |        |         |

| Events                   | Status | Details |
| ------------------------ | ------ | ------- |
| `input.fileDialogOpened` |        |         |

### The `webExtension` module

| Commands                 | Status | Details                                   |
| ------------------------ | ------ | ----------------------------------------- |
| `webExtension.install`   | ⛔     | Blocked by web extension not implemented. |
| `webExtension.uninstall` | ⛔     | Blocked by web extension not implemented. |
