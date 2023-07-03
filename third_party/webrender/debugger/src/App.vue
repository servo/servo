<template>
    <div>
        <app-navbar></app-navbar>
        <div class="section">
            <div class="container">
                <div class="columns">
                    <div class="column is-3">
                        <app-navmenu></app-navmenu>
                    </div>
                    <div class="column">
                        <app-options v-if="page == 'options'"></app-options>
                        <app-passview v-if="page == 'passes'"></app-passview>
                        <app-rendertaskview v-if="page == 'render_tasks'"></app-rendertaskview>
                        <app-documentview v-if="page == 'documents'"></app-documentview>
                        <app-clipscrolltreeview v-if="page == 'clip_scroll_tree'"></app-clipscrolltreeview>
                        <app-screenshotview v-if="page == 'screenshot'"></app-screenshotview>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script>
import NavBar from './components/NavBar.vue'
import NavMenu from './components/NavMenu.vue'
import OptionsPage from './components/OptionsPage.vue'
import PassViewPage from './components/PassViewPage.vue'
import RenderTaskViewPage from './components/RenderTaskViewPage.vue'
import DocumentViewPage from './components/DocumentViewPage.vue'
import ClipScrollTreeViewPage from './components/ClipScrollTreeViewPage.vue'
import ScreenshotPage from './components/ScreenshotPage.vue'

export default {
    name: 'app',
    components: {
        'app-navbar': NavBar,
        'app-navmenu': NavMenu,
        'app-options': OptionsPage,
        'app-passview': PassViewPage,
        'app-rendertaskview': RenderTaskViewPage,
        'app-documentview': DocumentViewPage,
        'app-clipscrolltreeview': ClipScrollTreeViewPage,
        'app-screenshotview': ScreenshotPage,
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
