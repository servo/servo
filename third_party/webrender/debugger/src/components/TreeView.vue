<template>
    <li>
        <div v-on:click="toggle">
            <span v-if="isFolder">[{{open ? '-' : '+'}}]</span>
            {{model.description}}
        </div>
        <ul style="padding-left: 1em; line-height: 1.5em;" v-show="open" v-if="isFolder">
            <treeview v-for="model in model.children" :model="model"></treeview>
        </ul>
    </li>
</template>

<script>
export default {
    name: 'treeview',
    props: [
        'model',
    ],
    data: function () {
        return {
            open: false
        }
    },
    computed: {
        isFolder: function () {
            return this.model.children && this.model.children.length
        }
    },
    methods: {
        toggle: function () {
            if (this.isFolder) {
                this.open = !this.open
            }
        },
    },
}
</script>

<style>
</style>
