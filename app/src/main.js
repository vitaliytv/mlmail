import '@quasar/extras/material-symbols-outlined/material-symbols-outlined.css'
import { Quasar, Dialog, Notify } from 'quasar'
import './app.scss'
import App from './App.vue'

createApp(App)
  .use(Quasar, {
    config: {
      dark: 'auto'
    },
    plugins: { Dialog, Notify },
    iconSet: 'material-symbols-outlined'
  })
  .mount('#app')
