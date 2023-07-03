<template>
    <div class="box">
        <h1 class="title">Passes <a :disabled="disabled" v-on:click="fetch" class="button is-info">Refresh</a></h1>
        <hr/>
        <div v-for="(pass, pass_index) in passes">
            <p class="has-text-black-bis">Pass {{pass_index}}</p>
            <div v-for="(target, target_index) in pass.targets">
                <p style="text-indent: 2em;" class="has-text-grey-dark">Target {{target_index}} ({{target.kind}})</p>
                <div v-for="(batch, batch_index) in target.batches">
                    <p style="text-indent: 4em;" class="has-text-grey">Batch {{batch_index}} ({{batch.description}}, {{batch.kind}}, {{batch.count}} instances)</p>
                </div>
            </div>
            <hr/>
        </div>
    </div>
</template>

<script>
export default {
    methods: {
        fetch: function() {
            this.$store.dispatch('sendMessage', "fetch_passes");
        }
    },
    computed: {
        disabled() {
            return !this.$store.state.connected
        },
        passes() {
            return this.$store.state.passes
        }
    },
}
</script>

<style>
</style>
