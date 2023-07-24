<template>
    <div class="box">
        <h1 class="title">Render Tasks <a :disabled="disabled" v-on:click="fetch" class="button is-info">Refresh</a></h1>
        <hr/>
        <div>
            <ul>
                <app-treeview :model=render_tasks></app-treeview>
            </ul>
        </div>
    </div>
</template>

<script>
import TreeView from './TreeView.vue'

export default {
    components: {
        'app-treeview': TreeView,
    },
    methods: {
        fetch: function() {
            this.$store.dispatch('sendMessage', "fetch_render_tasks");
        }
    },
    computed: {
        disabled() {
            return !this.$store.state.connected
        },
        render_tasks() {
            return this.$store.state.render_tasks
        }
    },
}
</script>

<style>
</style>
