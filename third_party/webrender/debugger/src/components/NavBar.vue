<template>
    <nav class="navbar has-shadow">
          <div class="navbar-brand">
            <a class="navbar-item" href="#">WebRender Debugger</a>
          </div>

        <div class="navbar-menu">
            <div class="navbar-start"></div>

            <div class="navbar-end">
                <div class="navbar-item">
                    <p class="control">
                        <button v-if="isConnected" @click="disconnect" class="button is-danger">Disconnect</button>
                        <button v-else @click="connect" class="button is-success">Connect</button>
                    </p>
                </div>
            </div>
        </div>
    </nav>
</template>

<script>
export default {
    computed: {
        isConnected() {
            return this.$store.state.connected;
        },
    },
    methods: {
        connect() {
            this.$store.dispatch('connect');
        },
        disconnect() {
            this.$store.dispatch('disconnect');
        },
    }
}
</script>

<style>
</style>
