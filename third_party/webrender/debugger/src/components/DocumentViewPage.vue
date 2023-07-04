<template>
    <div class="box">
        <h1 class="title">Documents <a :disabled="disabled" v-on:click="fetch" class="button is-info">Refresh</a></h1>
        <hr/>
        <div>
            <ul>
                <app-treeview :model=documents></app-treeview>
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
            this.$store.dispatch('sendMessage', "fetch_documents");
        }
    },
    computed: {
        disabled() {
            return !this.$store.state.connected
        },
        documents() {
            return this.$store.state.documents
        }
    },
}
</script>

<style>
</style>
