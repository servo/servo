<template>
    <div class="box">
        <h1 class="title">Clip-Scroll Tree <a :disabled="disabled" v-on:click="fetch" class="button is-info">Refresh</a></h1>
        <hr/>
        <div>
            <ul>
                <app-treeview :model=clip_scroll_tree></app-treeview>
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
            this.$store.dispatch('sendMessage', "fetch_clip_scroll_tree");
        }
    },
    computed: {
        disabled() {
            return !this.$store.state.connected
        },
        clip_scroll_tree() {
            return this.$store.state.clip_scroll_tree
        }
    },
}
</script>

<style>
</style>
