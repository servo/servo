Command Line Arguments
========================
# General

You can see available commands with:
```
./mach -h
./mach <sub-command> -h
```
Only arguments that need more explanation will be documented here.

# Run
## Enable Experimental Features
Use `--pref` to enable experimental features like experimental DOM API, JavaScript API and CSS properties.

e.g. To enable `flex` and `flex-direction` css properties:
```
./mach run -d -- --pref layout.flex.enabled --pref layout.flex-direction.enabled ...
```

You can find all the available preferences at [resources/prefs.json](http://mxr.mozilla.org/servo/source/resources/prefs.json).

# Debugging
## Remote Debugging
Use `--devtools 6000` to start the devtools server on port 6000.

e.g.
```
./mach run -d --devtools 6000 https://servo.org
```

To connect to the server, follow [this guide](https://developer.mozilla.org/en-US/docs/Tools/Remote_Debugging/Debugging_Firefox_Desktop#Connect).
