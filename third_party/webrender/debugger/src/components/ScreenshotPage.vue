<template>
    <div class="box">
        <h1 class="title">Screenshot <a :disabled="disabled" v-on:click="fetch" class="button is-info">Refresh</a></h1>
        <hr/>
        <div>
            <ul>
                <img v-if="screenshot.length > 0" style="transform: scaleY(-1); width: 1024px; height:768px" :src="'data:image/png;base64,' + screenshot" />
            </ul>
        </div>
    </div>
</template>

<script>
export default {
    computed: {
        disabled() {
            return !this.$store.state.connected
        },
        screenshot() {
            return this.$store.state.screenshot
        },
    },
    methods: {
        fetch: function() {
            this.$store.dispatch('sendMessage', "fetch_screenshot");
        }
    },
}
</script>

<style>
</style>
