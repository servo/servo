"use strict";

(function() {
    const pending = new Map();
    const event_target = new EventTarget();

    let result = null;
    let ctx_cmd_id = 0;
    let testharness_context = null;

    window.addEventListener("message", function(event) {
        const data = event.data;

        if (typeof data !== "object" && data !== null) {
            return;
        }

        if (is_test_context() && data.type === "testdriver-command") {
            const command = data.message;
            const ctx_id = command.cmd_id;
            delete command.cmd_id;
            const cmd_id = window.__wptrunner_message_queue.push(command);
            let on_success = (data) => {
                data.type = "testdriver-complete";
                data.cmd_id = ctx_id;
                event.source.postMessage(data, "*");
            };
            let on_failure = (data) => {
                data.type = "testdriver-complete";
                data.cmd_id = ctx_id;
                event.source.postMessage(data, "*");
            };
            pending.set(cmd_id, [on_success, on_failure]);
        } else if (data.type === "testdriver-complete") {
            const cmd_id = data.cmd_id;
            const [on_success, on_failure] = pending.get(cmd_id);
            pending.delete(cmd_id);
            const resolver = data.status === "success" ? on_success : on_failure;
            resolver(data);
            if (is_test_context()) {
                window.__wptrunner_process_next_event();
            }
        } else if (data.type === "testdriver-event") {
            const event_data = JSON.parse(data.message);
            const event_name = event_data.method;
            const event = new Event(event_name);
            event.payload = event_data.params;
            event_target.dispatchEvent(event);
        }
    });

    function is_test_context() {
      return window.__wptrunner_message_queue !== undefined;
    }

    // Code copied from /common/utils.js
    function rand_int(bits) {
        if (bits < 1 || bits > 53) {
            throw new TypeError();
        } else {
            if (bits >= 1 && bits <= 30) {
                return 0 | ((1 << bits) * Math.random());
            } else {
                var high = (0 | ((1 << (bits - 30)) * Math.random())) * (1 << 30);
                var low = 0 | ((1 << 30) * Math.random());
                return  high + low;
            }
        }
    }

    function to_hex(x, length) {
        var rv = x.toString(16);
        while (rv.length < length) {
            rv = "0" + rv;
        }
        return rv;
    }

    function get_window_id(win) {
        if (win == window && is_test_context()) {
            return null;
        }
        if (!win.__wptrunner_id) {
            // generate a uuid
            win.__wptrunner_id = [to_hex(rand_int(32), 8),
                                  to_hex(rand_int(16), 4),
                                  to_hex(0x4000 | rand_int(12), 4),
                                  to_hex(0x8000 | rand_int(14), 4),
                                  to_hex(rand_int(48), 12)].join("-");
        }
        return win.__wptrunner_id;
    }

    const get_context = function(element) {
        if (!element) {
            return null;
        }
        let elementWindow = element.ownerDocument.defaultView;
        if (!elementWindow) {
            throw new Error("Browsing context for element was detached");
        }
        return elementWindow;
    };

    const get_selector = function(element) {
        let selector;

        if (element.id) {
            const id = element.id;

            selector = "#";
            // escape everything, because it's easy to implement
            for (let i = 0, len = id.length; i < len; i++) {
                selector += '\\' + id.charCodeAt(i).toString(16) + ' ';
            }
        } else {
            // push and then reverse to avoid O(n) unshift in the loop
            let segments = [];
            for (let node = element;
                 node.parentElement;
                 node = node.parentElement) {
                let segment = "*|" + node.localName;
                let nth = Array.prototype.indexOf.call(node.parentElement.children, node) + 1;
                segments.push(segment + ":nth-child(" + nth + ")");
            }
            segments.push(":root");
            segments.reverse();

            selector = segments.join(" > ");
        }

        return selector;
    };

    /**
     * Create an action and return a promise that resolves when the action is complete.
     * @param name: The name of the action to create.
     * @param params: The properties to pass to the action.
     * @return {Promise<any>}: A promise that resolves with the action result when the action is complete.
     */
    const create_action = function(name, params) {
        let cmd_id;
        const action_msg = {type: "action",
                            action: name,
                            ...params};
        if (is_test_context()) {
            cmd_id = window.__wptrunner_message_queue.push(action_msg);
        } else {
            if (testharness_context === null) {
                throw new Error("Tried to run in a non-testharness window without a call to set_test_context");
            }
            cmd_id = ctx_cmd_id++;
            action_msg.cmd_id = cmd_id;
            window.test_driver.message_test({type: "testdriver-command",
                                             message: action_msg});
        }
        const pending_promise = new Promise(function(resolve, reject) {
            const on_success = data => {
                result = JSON.parse(data.message).result;
                resolve(result);
            };
            const on_failure = data => {
                reject(`${data.status}: ${data.message}`);
            };
            pending.set(cmd_id, [on_success, on_failure]);
        });
        return pending_promise;
    };

    /**
     * Create an action in a specific context and return a promise that
     * resolves when the action is complete. This is required for WebDriver
     * Classic actions, as they require a specific context.
     * @param name: The name of the action to create.
     * @param context: The context in which to run the action. `null` for the
     * current window.
     * @param params: The properties to pass to the action.
     * @return {Promise<any>}: A promise that resolves with the action result
     * when the action is complete.
     */
    const create_context_action = function (name, context, params) {
        const context_params = {...params};
        if (context) {
            context_params.context = get_window_id(context);
        }
        if (context === null && !is_test_context()) {
            context_params.context = get_window_id(window);
        }
        return create_action(name, context_params);
    };

    const subscribe = function (params) {
        return create_action("bidi.session.subscribe", {
            // Default to subscribing to the window's events.
            contexts: [window],
            ...params
        });
    };

    window.test_driver_internal.in_automation = true;

    window.test_driver_internal.bidi.log.entry_added.subscribe =
        function (params) {
            return subscribe({
                params,
                events: ["log.entryAdded"]
            })
        };

    window.test_driver_internal.bidi.log.entry_added.on = function (callback) {
        const on_event = (event) => {
            callback(event.payload);
        };
        event_target.addEventListener("log.entryAdded", on_event);
        return () => event_target.removeEventListener("log.entryAdded",
            on_event);
    };

    window.test_driver_internal.set_test_context = function(context) {
        if (window.__wptrunner_message_queue) {
            throw new Error("Tried to set testharness context in a window containing testharness.js");
        }
        testharness_context = context;
    };

    window.test_driver_internal.click = function(element) {
        const selector = get_selector(element);
        const context = get_context(element);
        return create_context_action("click", context, {selector});
    };

    window.test_driver_internal.delete_all_cookies = function(context=null) {
        return create_context_action("delete_all_cookies", context, {});
    };

    window.test_driver_internal.get_all_cookies = function(context=null) {
        return create_context_action("get_all_cookies", context, {});
    };

    window.test_driver_internal.get_computed_label = function(element) {
        const selector = get_selector(element);
        const context = get_context(element);
        return create_context_action("get_computed_label", context, {selector});
    };

    window.test_driver_internal.get_computed_role = function(element) {
        const selector = get_selector(element);
        const context = get_context(element);
        return create_context_action("get_computed_role", context, {selector});
    };

    window.test_driver_internal.get_named_cookie = function(name, context=null) {
        return create_context_action("get_named_cookie", context, {name});
    };

    window.test_driver_internal.minimize_window = function(context=null) {
        return create_context_action("minimize_window", context, {});
    };

    window.test_driver_internal.set_window_rect = function(rect, context=null) {
        return create_context_action("set_window_rect", context, {rect});
    };

    window.test_driver_internal.get_window_rect = function(context=null) {
        return create_context_action("get_window_rect", context, {});
    };

    window.test_driver_internal.send_keys = function(element, keys) {
        const selector = get_selector(element);
        const context = get_context(element);
        return create_context_action("send_keys", context, {selector, keys});
    };

    window.test_driver_internal.action_sequence = function(actions, context=null) {
        for (let actionSequence of actions) {
            if (actionSequence.type == "pointer") {
                for (let action of actionSequence.actions) {
                    // The origin of each action can only be an element or a string of a value "viewport" or "pointer".
                    if (action.type == "pointerMove" && typeof(action.origin) != 'string') {
                        let action_context = get_context(action.origin);
                        action.origin = {selector: get_selector(action.origin)};
                        if (context !== null && action_context !== context) {
                            throw new Error("Actions must be in a single context");
                        }
                        context = action_context;
                    }
                }
            }
        }
        return create_context_action("action_sequence", context, {actions});
    };

    window.test_driver_internal.generate_test_report = function(message, context=null) {
        return create_context_action("generate_test_report", context, {message});
    };

    window.test_driver_internal.set_permission = function(permission_params, context=null) {
        return create_context_action("set_permission", context, {permission_params});
    };

    window.test_driver_internal.add_virtual_authenticator = function(config, context=null) {
        return create_context_action("add_virtual_authenticator", context, {config});
    };

    window.test_driver_internal.remove_virtual_authenticator = function(authenticator_id, context=null) {
        return create_context_action("remove_virtual_authenticator", context, {authenticator_id});
    };

    window.test_driver_internal.add_credential = function(authenticator_id, credential, context=null) {
        return create_context_action("add_credential", context, {authenticator_id, credential});
    };

    window.test_driver_internal.get_credentials = function(authenticator_id, context=null) {
        return create_context_action("get_credentials", context, {authenticator_id});
    };

    window.test_driver_internal.remove_credential = function(authenticator_id, credential_id, context=null) {
        return create_context_action("remove_credential", context, {authenticator_id, credential_id});
    };

    window.test_driver_internal.remove_all_credentials = function(authenticator_id, context=null) {
        return create_context_action("remove_all_credentials", context, {authenticator_id});
    };

    window.test_driver_internal.set_user_verified = function(authenticator_id, uv, context=null) {
        return create_context_action("set_user_verified", context, {authenticator_id, uv});
    };

    window.test_driver_internal.set_spc_transaction_mode = function(mode, context = null) {
        return create_context_action("set_spc_transaction_mode", context, {mode});
    };

    window.test_driver_internal.set_rph_registration_mode = function(mode, context = null) {
        return create_context_action("set_rph_registration_mode", context, {mode});
    };

    window.test_driver_internal.cancel_fedcm_dialog = function(context = null) {
        return create_context_action("cancel_fedcm_dialog", context, {});
    };

    window.test_driver_internal.click_fedcm_dialog_button = function(dialog_button, context = null) {
        return create_context_action("click_fedcm_dialog_button", context, {dialog_button});
    };

    window.test_driver_internal.select_fedcm_account = function(account_index, context = null) {
        return create_context_action("select_fedcm_account", context, {account_index});
    };

    window.test_driver_internal.get_fedcm_account_list = function(context = null) {
        return create_context_action("get_fedcm_account_list", context, {});
    };

    window.test_driver_internal.get_fedcm_dialog_title = function(context = null) {
        return create_context_action("get_fedcm_dialog_title", context, {});
    };

    window.test_driver_internal.get_fedcm_dialog_type = function(context = null) {
        return create_context_action("get_fedcm_dialog_type", context, {});
    };

    window.test_driver_internal.set_fedcm_delay_enabled = function(enabled, context = null) {
        return create_context_action("set_fedcm_delay_enabled", context, {enabled});
    };

    window.test_driver_internal.reset_fedcm_cooldown = function(context = null) {
        return create_context_action("reset_fedcm_cooldown", context, {});
    };

    window.test_driver_internal.create_virtual_sensor = function(sensor_type, sensor_params={}, context=null) {
        return create_context_action("create_virtual_sensor", context, {sensor_type, sensor_params});
    };

    window.test_driver_internal.update_virtual_sensor = function(sensor_type, reading, context=null) {
        return create_context_action("update_virtual_sensor", context, {sensor_type, reading});
    };

    window.test_driver_internal.remove_virtual_sensor = function(sensor_type, context=null) {
        return create_context_action("remove_virtual_sensor", context, {sensor_type});
    };

    window.test_driver_internal.get_virtual_sensor_information = function(sensor_type, context=null) {
        return create_context_action("get_virtual_sensor_information", context, {sensor_type});
    };

    window.test_driver_internal.set_device_posture = function(posture, context=null) {
        return create_context_action("set_device_posture", context, {posture});
    };

    window.test_driver_internal.clear_device_posture = function(context=null) {
        return create_context_action("clear_device_posture", context, {});
    };

    window.test_driver_internal.run_bounce_tracking_mitigations = function (context = null) {
        return create_context_action("run_bounce_tracking_mitigations", context, {});
    };

    window.test_driver_internal.create_virtual_pressure_source = function(source_type, metadata={}, context=null) {
        return create_context_action("create_virtual_pressure_source", context, {source_type, metadata});
    };

    window.test_driver_internal.update_virtual_pressure_source = function(source_type, sample, context=null) {
        return create_context_action("update_virtual_pressure_source", context, {source_type, sample});
    };

    window.test_driver_internal.remove_virtual_pressure_source = function(source_type, context=null) {
        return create_context_action("remove_virtual_pressure_source", context, {source_type});
    };
})();
