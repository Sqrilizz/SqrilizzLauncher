import '@fortawesome/fontawesome-free/css/all.min.css'
import 'floating-vue/dist/style.css'

import { VueQueryPlugin } from '@tanstack/vue-query'
import { createPlugin } from '@vintl/vintl/plugin'
import FloatingVue from 'floating-vue'
import { createPinia } from 'pinia'
import { createApp } from 'vue'

import App from '@/App.vue'
import router from '@/routes'

const VIntlPlugin = createPlugin({
	controllerOpts: {
		defaultLocale: 'en-US',
		locale: 'en-US',
		locales: [
			{
				tag: 'en-US',
				meta: {
					displayName: 'American English',
				},
			},
		],
	},
	globalMixin: true,
	injectInto: [],
})

const pinia = createPinia()

let app = createApp(App)

app.use(VueQueryPlugin)
app.use(router)
app.use(pinia)
app.use(FloatingVue, {
	themes: {
		'ribbit-popout': {
			$extend: 'dropdown',
			placement: 'bottom-end',
			instantMove: true,
			distance: 8,
		},
	},
})
app.use(VIntlPlugin)

app.mount('#app')
