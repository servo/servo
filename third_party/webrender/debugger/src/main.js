import Vue from 'vue';
import Buefy from 'buefy';
import 'buefy/dist/buefy.css';
import "vue-material-design-icons/styles.css";
import App from './App.vue';
import store from './store';

Vue.use(Buefy);

new Vue({
    el: '#app',
    store,
    render: h => h(App)
})
