<template>
    <aside class="menu">
        <p class="menu-label">
            Pages
        </p>
        <ul class="menu-list">
            <li><a @click="setPage('options')" :class="{ 'is-active': page == 'options' }">Debug Options</a></li>
            <li><a @click="setPage('passes')" :class="{ 'is-active': page == 'passes' }">Passes</a></li>
            <li><a @click="setPage('render_tasks')" :class="{ 'is-active': page == 'render_tasks' }">Render Tasks</a></li>
            <li><a @click="setPage('documents')" :class="{ 'is-active': page == 'documents' }">Documents</a></li>
            <li><a @click="setPage('clip_scroll_tree')" v-bind:class="{ 'is-active': page == 'clip_scroll_tree' }">Clip-Scroll Tree</a></li>
            <li><a @click="setPage('screenshot')" v-bind:class="{ 'is-active': page == 'screenshot' }">Screenshot</a></li>
        </ul>
    </aside>
</template>

<script>
export default {
    methods: {
        setPage(name) {
            this.$store.commit('setPage', name);
        },
    },
    computed: {
        page() {
            return this.$store.state.page;
        }
    },
}
</script>

<style>
</style>
