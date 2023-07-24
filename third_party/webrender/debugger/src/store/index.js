import Vue from 'vue'
import Vuex from 'vuex'

Vue.use(Vuex)

class Connection {
    constructor() {
        this.ws = null;
    }

    connect(context) {
        var ws = new WebSocket("ws://127.0.0.1:3583");

        ws.onopen = function() {
            context.commit('setConnected', true);
        }

        ws.onmessage = function(evt) {
            var json = JSON.parse(evt.data);
            if (json['kind'] == "passes") {
                context.commit('setPasses', json['passes']);
            } else if (json['kind'] == "render_tasks") {
                context.commit('setRenderTasks', json['root']);
            } else if (json['kind'] == "documents") {
                context.commit('setDocuments', json['root']);
            } else if (json['kind'] == "clip_scroll_tree") {
                context.commit('setClipScrollTree', json['root']);
            } else if (json['kind'] == "screenshot") {
                context.commit('setScreenshot', json['data']);
            } else {
                console.warn("unknown message kind: " + json['kind']);
            }
        }

        ws.onclose = function() {
            context.commit('setConnected', false);
        }

        this.ws = ws;
    }

    send(msg) {
        if (this.ws !== null) {
            this.ws.send(msg);
        }
    }

    disconnect() {
        if (this.ws !== null) {
            this.ws.close();
            this.ws = null;
        }
    }
}

var connection = new Connection();

const store = new Vuex.Store({
    strict: true,
    state: {
        connected: false,
        page: 'options',
        passes: [],
        render_tasks: [],
        documents: [],
        clip_scroll_tree: [],
        screenshot: [],
    },
    mutations: {
        setConnected(state, connected) {
            state.connected = connected;
        },
        setPage(state, name) {
            state.page = name;
        },
        setPasses(state, passes) {
            state.passes = passes;
        },
        setRenderTasks(state, render_tasks) {
            state.render_tasks = render_tasks;
        },
        setDocuments(state, documents) {
            state.documents = documents;
        },
        setClipScrollTree(state, clip_scroll_tree) {
            state.clip_scroll_tree = clip_scroll_tree;
        },
        setScreenshot(state, screenshot) {
            state.screenshot = screenshot;
        },
    },
    actions: {
        connect(context) {
            connection.connect(context);
        },
        disconnect(context) {
            connection.disconnect();
        },
        sendMessage(context, msg) {
            connection.send(msg);
        }
    }
});

export default store;
