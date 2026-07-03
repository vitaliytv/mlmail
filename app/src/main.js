import '@quasar/extras/material-symbols-outlined/material-symbols-outlined.css'
import { Quasar, Notify } from 'quasar'
import 'quasar/src/css/index.sass'
import App from './App.vue'

createApp(App)
  .use(Quasar, {
    config: {
      dark: 'auto'
    },
    plugins: { Notify },
    iconSet: 'material-symbols-outlined'
  })
  .mount('#app')
