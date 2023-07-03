<template>
    <div class="box">
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setProfiler($event.target.checked)">
                Profiler
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setTextureCacheDebugger($event.target.checked)">
                Texture cache debugger
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setRenderTargetDebugger($event.target.checked)">
                Render target debugger
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setAlphaRectsDebugger($event.target.checked)">
                Alpha primitive rects debugger
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setGpuTimeQueries($event.target.checked)">
                Enable GPU time queries
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setGpuSampleQueries($event.target.checked)">
                Enable GPU sample queries
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setOpaquePass(!$event.target.checked)">
                Disable opaque pass
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setAlphaPass(!$event.target.checked)">
                Disable alpha pass
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setClipMasks(!$event.target.checked)">
                Disable clip masks
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setTextPrims(!$event.target.checked)">
                Disable text primitives
            </label>
        </div>
        <div class="field">
            <label class="checkbox">
                <input type="checkbox" :disabled="disabled" v-on:click="setGradientPrims(!$event.target.checked)">
                Disable gradient primitives
            </label>
        </div>
    </div>
</template>

<script>
export default {
    computed: {
        disabled() {
            return !this.$store.state.connected
        }
    },
    methods: {
        setProfiler(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_profiler");
            } else {
                this.$store.dispatch('sendMessage', "disable_profiler");
            }
        },
        setTextureCacheDebugger(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_texture_cache_debug");
            } else {
                this.$store.dispatch('sendMessage', "disable_texture_cache_debug");
            }
        },
        setRenderTargetDebugger(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_render_target_debug");
            } else {
                this.$store.dispatch('sendMessage', "disable_render_target_debug");
            }
        },
        setAlphaRectsDebugger(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_alpha_rects_debug");
            } else {
                this.$store.dispatch('sendMessage', "disable_alpha_rects_debug");
            }
        },
        setGpuTimeQueries(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_gpu_time_queries");
            } else {
                this.$store.dispatch('sendMessage', "disable_gpu_time_queries");
            }
        },
        setGpuSampleQueries(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_gpu_sample_queries");
            } else {
                this.$store.dispatch('sendMessage', "disable_gpu_sample_queries");
            }
        },
        setOpaquePass(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_opaque_pass");
            } else {
                this.$store.dispatch('sendMessage', "disable_opaque_pass");
            }
        },
        setAlphaPass(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_alpha_pass");
            } else {
                this.$store.dispatch('sendMessage', "disable_alpha_pass");
            }
        },
        setClipMasks(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_clip_masks");
            } else {
                this.$store.dispatch('sendMessage', "disable_clip_masks");
            }
        },
        setTextPrims(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_text_prims");
            } else {
                this.$store.dispatch('sendMessage', "disable_text_prims");
            }
        },
        setGradientPrims(enabled) {
            if (enabled) {
                this.$store.dispatch('sendMessage', "enable_gradient_prims");
            } else {
                this.$store.dispatch('sendMessage', "disable_gradient_prims");
            }
        }
    },
}
</script>

<style>
</style>
