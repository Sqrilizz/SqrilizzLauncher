<script setup>
import { AuthFeature, TauriModrinthClient } from '@modrinth/api-client'
import {
    CompassIcon,
    DownloadIcon,
    ExternalIcon,
    HomeIcon,
    LeftArrowIcon,
    LibraryIcon,
    LogOutIcon,
    MaximizeIcon,
    MinimizeIcon,
    NewspaperIcon,
    NotepadTextIcon,
    PlusIcon,
    RefreshCwIcon,
    RestoreIcon,
    RightArrowIcon,
    ServerIcon,
    SettingsIcon,
    UserIcon,
    WorldIcon,
    XIcon
} from '@modrinth/assets'
import {
    Admonition,
    Avatar,
    Button,
    ButtonStyled,
    commonMessages,
    NewsArticleCard,
    NotificationPanel,
    OverflowMenu,
    ProgressSpinner,
    provideModrinthClient,
    provideNotificationManager,
    useDebugLogger,
} from '@modrinth/ui'
import { renderString } from '@modrinth/utils'
import { useQuery } from '@tanstack/vue-query'
import { getVersion } from '@tauri-apps/api/app'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { openUrl } from '@tauri-apps/plugin-opener'
import { type } from '@tauri-apps/plugin-os'
import { defineMessages, useVIntl } from '@vintl/vintl'
import { $fetch } from 'ofetch'
import { computed, onMounted, onUnmounted, provide, ref } from 'vue'
import { RouterView, useRoute, useRouter } from 'vue-router'

import ModrinthLoadingIndicator from '@/components/LoadingIndicatorBar.vue'
import AccountsCard from '@/components/ui/AccountsCard.vue'
import Breadcrumbs from '@/components/ui/Breadcrumbs.vue'
import ErrorModal from '@/components/ui/ErrorModal.vue'
// [AR] Patch: Removed FriendsList import
import IncompatibilityWarningModal from '@/components/ui/install_flow/IncompatibilityWarningModal.vue'
import InstallConfirmModal from '@/components/ui/install_flow/InstallConfirmModal.vue'
import ModInstallModal from '@/components/ui/install_flow/ModInstallModal.vue'
import InstanceCreationModal from '@/components/ui/InstanceCreationModal.vue'
import AppSettingsModal from '@/components/ui/modal/AppSettingsModal.vue'
import NavButton from '@/components/ui/NavButton.vue'
import QuickInstanceSwitcher from '@/components/ui/QuickInstanceSwitcher.vue'
import RunningAppBar from '@/components/ui/RunningAppBar.vue'
import SplashScreen from '@/components/ui/SplashScreen.vue'
import URLConfirmModal from '@/components/ui/URLConfirmModal.vue'
import { useCheckDisableMouseover } from '@/composables/macCssFix.js'
import { debugAnalytics, optOutAnalytics, trackEvent } from '@/helpers/analytics'
import { check_reachable } from '@/helpers/auth.js'
import { get_user } from '@/helpers/cache.js'
import { command_listener, warning_listener } from '@/helpers/events.js'
import { useFetch } from '@/helpers/fetch.js'
// [AR] Patch: Removed Modrinth auth imports
import { list } from '@/helpers/profile.js'
import { get as getSettings, set as setSettings } from '@/helpers/settings.ts'
import { get_opening_command, initialize_state } from '@/helpers/state'
import {
    getOS,
    isDev
} from '@/helpers/utils.js'
import {
    provideAppUpdateDownloadProgress
} from '@/providers/download-progress.ts'
import { useError } from '@/store/error.js'
import { useInstall } from '@/store/install.js'
import { useLoading, useTheming } from '@/store/state'

import { create_profile_and_install_from_file } from './helpers/pack'
import { AppNotificationManager } from './providers/app-notifications'

// [AR] Imports
import { get, set } from '@/helpers/settings.ts'
import { getRemote, updateState } from '@/helpers/update.js'

const themeStore = useTheming()

const notificationManager = new AppNotificationManager()
provideNotificationManager(notificationManager)
const { handleError, addNotification } = notificationManager

const tauriApiClient = new TauriModrinthClient({
	userAgent: `modrinth/theseus/${getVersion()} (support@modrinth.com)`,
	features: [
		new AuthFeature({
			token: async () => (await getCreds()).session,
		}),
	],
})
provideModrinthClient(tauriApiClient)

const news = ref([])
const availableSurvey = ref(false)

// Load news immediately using Tauri HTTP (bypasses CSP)
setTimeout(async () => {
	const NEWS_API_URL = 'http://151.241.155.196:3000'

	// First, load from cache
	try {
		const cacheContent = await invoke('read_news_cache')
		const cacheData = JSON.parse(cacheContent)
		if (cacheData && cacheData.articles && cacheData.articles.length > 0) {
			news.value = cacheData.articles
				.map((article) => ({
					...article,
					path: article.link,
					thumbnail: article.thumbnail,
					title: article.title,
					summary: article.summary,
					date: article.date,
				}))
				.slice(0, 4)
		}
	} catch (err) {
		// Cache not available
	}

	// Then, fetch from API using Tauri HTTP
	try {
		const apiData = await invoke('fetch_news_from_api')

		const parsedData = JSON.parse(apiData)
		if (parsedData && parsedData.articles && parsedData.articles.length > 0) {
			// Save to cache
			try {
				await invoke('write_news_cache', {
					content: apiData
				})
			} catch (writeErr) {
				// Could not save to cache
			}

			// Update display
			news.value = parsedData.articles
				.map((article) => ({
					...article,
					path: article.link,
					thumbnail: article.thumbnail,
					title: article.title,
					summary: article.summary,
					date: article.date,
				}))
				.slice(0, 4)
		}
	} catch (err) {
		// API request failed, using cached news if available
	}
}, 2000)

const urlModal = ref(null)
const installationModal = ref(null)
const settingsModal = ref(null)

const offline = ref(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

const showOnboarding = ref(false)
const nativeDecorations = ref(false)

const os = ref('')

const stateInitialized = ref(false)

const criticalErrorMessage = ref()

const isMaximized = ref(false)

// Linux warning banner
const showLinuxWarning = ref(false)
const linuxWarningDismissed = ref(false)

const authUnreachableDebug = useDebugLogger('AuthReachableChecker')
const authServerQuery = useQuery({
	queryKey: ['authServerReachability'],
	queryFn: async () => {
		await check_reachable()
		authUnreachableDebug('Auth servers are reachable')
		return true
	},
	refetchInterval: 5 * 60 * 1000, // 5 minutes
	retry: false,
	refetchOnWindowFocus: false,
})

const authUnreachable = computed(() => {
	if (authServerQuery.isError.value && !authServerQuery.isLoading.value) {
		console.warn('Failed to reach auth servers', authServerQuery.error.value)
		return true
	}
	return false
})

// Linux warning dismiss function
async function dismissLinuxWarning() {
	showLinuxWarning.value = false
	linuxWarningDismissed.value = true

	const settings = await getSettings()
	settings.linux_warning_dismissed = true
	await setSettings(settings)
}

// Function to load news from API and cache
function loadNews() {
	const NEWS_API_URL = 'http://151.241.155.196:3000/api/launchernews'

	// First, load from cache immediately for instant display
	invoke('read_news_cache')
		.then((cacheContent) => {
			try {
				const cacheData = JSON.parse(cacheContent)
				if (cacheData && cacheData.articles && cacheData.articles.length > 0) {
					news.value = cacheData.articles
						.map((article) => ({
							...article,
							path: article.link,
							thumbnail: article.thumbnail,
							title: article.title,
							summary: article.summary,
							date: article.date,
						}))
						.slice(0, 4)
				}
			} catch (e) {
				// Failed to parse cache
			}
		})
		.catch((err) => {
			// No cache available
		})

	// Then, fetch from API in background and update cache
	$fetch(NEWS_API_URL)
		.then(async (apiData) => {
			if (apiData && apiData.articles && apiData.articles.length > 0) {
				// Save to cache
				try {
					await invoke('write_news_cache', {
						content: JSON.stringify(apiData, null, 2)
					})
				} catch (writeErr) {
					// Could not save to cache
				}

				// Update display with fresh API data
				news.value = apiData.articles
					.map((article) => ({
						...article,
						path: article.link,
						thumbnail: article.thumbnail,
						title: article.title,
						summary: article.summary,
						date: article.date,
					}))
					.slice(0, 4)
			}
		})
		.catch((err) => {
			// API request failed, using cached news if available
		})
}

onMounted(async () => {
	await useCheckDisableMouseover()
	await getRemote(false) // Check for updates

	document.querySelector('body').addEventListener('click', handleClick)
	document.querySelector('body').addEventListener('auxclick', handleAuxClick)

	// Load news on mount
	loadNews()
})

onUnmounted(async () => {
	document.querySelector('body').removeEventListener('click', handleClick)
	document.querySelector('body').removeEventListener('auxclick', handleAuxClick)
})

const { formatMessage } = useVIntl()
const messages = defineMessages({
	updateInstalledToastTitle: {
		id: 'app.update.complete-toast.title',
		defaultMessage: 'Version {version} was successfully installed!',
	},
	updateInstalledToastText: {
		id: 'app.update.complete-toast.text',
		defaultMessage: 'Click here to view the changelog.',
	},
	reloadToUpdate: {
		id: 'app.update.reload-to-update',
		defaultMessage: 'Reload to install update',
	},
	downloadUpdate: {
		id: 'app.update.download-update',
		defaultMessage: 'Download update',
	},
	downloadingUpdate: {
		id: 'app.update.downloading-update',
		defaultMessage: 'Downloading update ({percent}%)',
	},
	authUnreachableHeader: {
		id: 'app.auth-servers.unreachable.header',
		defaultMessage: 'Cannot reach authentication servers',
	},
	authUnreachableBody: {
		id: 'app.auth-servers.unreachable.body',
		defaultMessage:
			'Minecraft authentication servers may be down right now. Check your internet connection and try again later.',
	},
})

async function setupApp() {
	// [AR] Patched
	const settings = await get()
  	settings.personalized_ads = false
  	settings.telemetry = false
  	await set(settings)

	stateInitialized.value = true
	const {
		native_decorations,
		theme,
		telemetry,
		personalized_ads,
		collapsed_navigation,
		advanced_rendering,
		onboarded,
		default_page,
		toggle_sidebar,
		developer_mode,
		feature_flags,
		pending_update_toast_for_version,
	} = await getSettings()

	if (default_page === 'Library') {
		await router.push('/library')
	}

	os.value = await getOS()
	const dev = await isDev()
	const version = await getVersion()
	showOnboarding.value = !onboarded

	// Check if Linux and warning not dismissed
	if (os.value === 'Linux') {
		const settings = await getSettings()
		linuxWarningDismissed.value = settings.linux_warning_dismissed || false
		showLinuxWarning.value = !linuxWarningDismissed.value
	}

	nativeDecorations.value = native_decorations
	if (os.value !== 'MacOS') await getCurrentWindow().setDecorations(native_decorations)

	themeStore.setThemeState(theme)
	themeStore.collapsedNavigation = collapsed_navigation
	themeStore.advancedRendering = advanced_rendering
	themeStore.toggleSidebar = toggle_sidebar
	themeStore.devMode = developer_mode
	themeStore.featureFlags = feature_flags

	isMaximized.value = await getCurrentWindow().isMaximized()

	await getCurrentWindow().onResized(async () => {
		isMaximized.value = await getCurrentWindow().isMaximized()
	})

	// [AR] Patched
	if (!telemetry) {
  	  console.info("[AR] • Telemetry disabled by default (Hard patched).")
  	  optOutAnalytics()
  	}
  	if (!personalized_ads) {
  	  console.info("[AR] • Personalized ads disabled by default (Hard patched).")
  	}
	if (dev) debugAnalytics()
	trackEvent('Launched', { version, dev, onboarded })

	if (!dev) document.addEventListener('contextmenu', (event) => event.preventDefault())

	const osType = await type()
	if (osType === 'macos') {
		document.getElementsByTagName('html')[0].classList.add('mac')
	} else {
		document.getElementsByTagName('html')[0].classList.add('windows')
	}

	await warning_listener((e) =>
		addNotification({
			title: 'Warning',
			text: e.message,
			type: 'warn',
		}),
	)

	useFetch(
		`https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
		'criticalAnnouncements',
		true,
	)
		.then((response) => response.json())
		.then((res) => {
			if (res && res.header && res.body) {
				criticalErrorMessage.value = res
			}
		})
		.catch(() => {
			console.log(
				`No critical announcement found at https://api.modrinth.com/appCriticalAnnouncement.json?version=${version}`,
			)
		})

	get_opening_command().then(handleCommand)
	// [AR] Patch: Removed fetchCredentials() call

	// Check for running Minecraft sessions and restore them
	try {
		const runningSessions = await check_running_sessions()
		if (runningSessions && runningSessions.length > 0) {
			console.info(`Restored ${runningSessions.length} running Minecraft session(s)`)
			addNotification({
				title: 'Sessions Restored',
				text: `Found ${runningSessions.length} running Minecraft instance(s)`,
				type: 'success',
			})
		}
	} catch (error) {
		console.warn('Failed to check for running sessions:', error)
	}

	// Skin system disabled to prevent crashes
	// try {
	// 	const skins = (await get_available_skins()) ?? []
	// 	const capes = (await get_available_capes()) ?? []
	// 	generateSkinPreviews(skins, capes)
	// } catch (error) {
	// 	console.warn('Failed to generate skin previews in app setup.', error)
	// }

	if (pending_update_toast_for_version !== null) {
		const settings = await getSettings()
		settings.pending_update_toast_for_version = null
		await setSettings(settings)
	}

	if (osType === 'windows') {
		await processPendingSurveys()
	} else {
		console.info('Skipping user surveys on non-Windows platforms')
	}
}

const stateFailed = ref(false)
initialize_state()
	.then(() => {
		setupApp().catch((err) => {
			stateFailed.value = true
			console.error(err)
			error.showError(err, null, false, 'state_init')
		})
	})
	.catch((err) => {
		stateFailed.value = true
		console.error('Failed to initialize app', err)
		error.showError(err, null, false, 'state_init')
	})

const handleClose = async () => {
	// [AR] Patch: window-state plugin disabled, just close window
	// await saveWindowState(StateFlags.ALL)
	await getCurrentWindow().close()
}

const router = useRouter()
router.afterEach((to, from, failure) => {
	trackEvent('PageView', {
		path: to.path,
		fromPath: from.path,
		failed: failure,
	})
})
const route = useRoute()

const loading = useLoading()
loading.setEnabled(false)

const error = useError()
const errorModal = ref()

const install = useInstall()
const modInstallModal = ref()
const installConfirmModal = ref()
const incompatibilityWarningModal = ref()

const credentials = ref()

const modrinthLoginFlowWaitModal = ref()

async function fetchCredentials() {
	const creds = await getCreds().catch(handleError)
	if (creds && creds.user_id) {
		creds.user = await get_user(creds.user_id).catch(handleError)
	}
	credentials.value = creds ?? null
}

async function signIn() {
	modrinthLoginFlowWaitModal.value.show()

	try {
		await login()
		await fetchCredentials()
	} catch (error) {
		if (
			typeof error === 'object' &&
			typeof error['message'] === 'string' &&
			error.message.includes('Login canceled')
		) {
			// Not really an error due to being a result of user interaction, show nothing
		} else {
			handleError(error)
		}
	} finally {
		modrinthLoginFlowWaitModal.value.hide()
	}
}

async function logOut() {
	await logout().catch(handleError)
	await fetchCredentials()
}

const MIDAS_BITFLAG = 1 << 0
const hasPlus = computed(
	() =>
		credentials.value &&
		credentials.value.user &&
		(credentials.value.user.badges & MIDAS_BITFLAG) === MIDAS_BITFLAG,
)

const sidebarToggled = ref(true)

themeStore.$subscribe(() => {
	sidebarToggled.value = !themeStore.toggleSidebar
})

const forceSidebar = computed(
	() => route.path.startsWith('/browse') || route.path.startsWith('/project'),
)
const sidebarVisible = computed(() => sidebarToggled.value || forceSidebar.value)

onMounted(() => {
	invoke('show_window')

	error.setErrorModal(errorModal.value)

	install.setIncompatibilityWarningModal(incompatibilityWarningModal)
	install.setInstallConfirmModal(installConfirmModal)
	install.setModInstallModal(modInstallModal)
})

const accounts = ref(null)
provide('accountsCard', accounts)

command_listener(handleCommand)
async function handleCommand(e) {
	if (!e) return

	if (e.event === 'RunMRPack') {
		// RunMRPack should directly install a local mrpack given a path
		if (e.path.endsWith('.mrpack')) {
			await create_profile_and_install_from_file(e.path).catch(handleError)
			trackEvent('InstanceCreate', {
				source: 'CreationModalFileDrop',
			})
		}
	} else {
		// Other commands are URL-based (deep linking)
		urlModal.value.show(e)
	}
}

const appUpdateDownload = {
	progress: ref(0),
	version: ref(),
}

function handleClick(e) {
	let target = e.target
	while (target != null) {
		if (target.matches('a')) {
			if (
				target.href &&
				['http://', 'https://', 'mailto:', 'tel:'].some((v) => target.href.startsWith(v)) &&
				!target.classList.contains('router-link-active') &&
				!target.href.startsWith('http://localhost') &&
				!target.href.startsWith('https://tauri.localhost') &&
				!target.href.startsWith('http://tauri.localhost')
			) {
				openUrl(target.href)
			}
			e.preventDefault()
			break
		}
		target = target.parentElement
	}
}

function handleAuxClick(e) {
	// disables middle click -> new tab
	if (e.button === 1) {
		e.preventDefault()
		// instead do a left click
		const event = new MouseEvent('click', {
			view: window,
			bubbles: true,
			cancelable: true,
		})
		e.target.dispatchEvent(event)
	}
}

function cleanupOldSurveyDisplayData() {
	const threeWeeksAgo = new Date()
	threeWeeksAgo.setDate(threeWeeksAgo.getDate() - 21)

	for (let i = 0; i < localStorage.length; i++) {
		const key = localStorage.key(i)

		if (key.startsWith('survey-') && key.endsWith('-display')) {
			const dateValue = new Date(localStorage.getItem(key))
			if (dateValue < threeWeeksAgo) {
				localStorage.removeItem(key)
			}
		}
	}
}

async function openSurvey() {
	if (!availableSurvey.value) {
		console.error('No survey to open')
		return
	}

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const formId = availableSurvey.value.tally_id

	const popupOptions = {
		layout: 'modal',
		width: 700,
		autoClose: 2000,
		hideTitle: true,
		hiddenFields: {
			user_id: userId,
		},
		onOpen: () => console.info('Opened user survey'),
		onClose: () => {
			console.info('Closed user survey')
		},
		onSubmit: () => console.info('Active user survey submitted'),
	}

	try {
		if (window.Tally?.openPopup) {
			console.info(`Opening Tally popup for user survey (form ID: ${formId})`)
			dismissSurvey()
			window.Tally.openPopup(formId, popupOptions)
		} else {
			console.warn('Tally script not yet loaded')
		}
	} catch (e) {
		console.error('Error opening Tally popup:', e)
	}

	console.info(`Found user survey to show with tally_id: ${formId}`)
	window.Tally.openPopup(formId, popupOptions)
}

function dismissSurvey() {
	localStorage.setItem(`survey-${availableSurvey.value.id}-display`, new Date())
	availableSurvey.value = undefined
}

async function processPendingSurveys() {
	function isWithinLastTwoWeeks(date) {
		const twoWeeksAgo = new Date()
		twoWeeksAgo.setDate(twoWeeksAgo.getDate() - 14)
		return date >= twoWeeksAgo
	}

	cleanupOldSurveyDisplayData()

	const creds = await getCreds().catch(handleError)
	const userId = creds?.user_id

	const instances = await list().catch(handleError)
	const isActivePlayer =
		instances.findIndex(
			(instance) =>
				isWithinLastTwoWeeks(instance.last_played) && !isWithinLastTwoWeeks(instance.created),
		) >= 0

	let surveys = []
	try {
		surveys = await $fetch('https://api.modrinth.com/v2/surveys')
	} catch (e) {
		console.error('Error fetching surveys:', e)
	}

	const surveyToShow = surveys.find(
		(survey) =>
			!!(
				localStorage.getItem(`survey-${survey.id}-display`) === null &&
				survey.type === 'tally_app' &&
				((survey.condition === 'active_player' && isActivePlayer) ||
					(survey.assigned_users?.includes(userId) && !survey.dismissed_users?.includes(userId)))
			),
	)

	if (surveyToShow) {
		availableSurvey.value = surveyToShow
	} else {
		console.info('No user survey to show')
	}
}

provideAppUpdateDownloadProgress(appUpdateDownload) // [AR Note] If delete this shit line -> SettingsModal will not work.
</script>

<template>
	<SplashScreen v-if="!stateFailed" ref="splashScreen" data-tauri-drag-region />
	<div id="teleports"></div>

	<!-- Linux Warning Modal -->
	<div
		v-if="showLinuxWarning && stateInitialized"
		class="fixed inset-0 z-[9999] flex items-center justify-center bg-black bg-opacity-50"
		@click.self="dismissLinuxWarning"
	>
		<div class="bg-bg-raised rounded-xl shadow-2xl max-w-md w-full mx-4 overflow-hidden">
			<div class="bg-orange-500 px-6 py-4 flex items-center gap-3">
				<svg xmlns="http://www.w3.org/2000/svg" class="h-8 w-8 text-white flex-shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
					<path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
				</svg>
				<h2 class="text-xl font-bold text-white">Linux Compatibility Notice</h2>
			</div>

			<div class="p-6 space-y-4">
				<p class="text-contrast">
					You are running Sqrilizz Launcher on Linux. Due to WebKit limitations, some features may not work as expected:
				</p>

				<ul class="list-disc list-inside space-y-2 text-secondary">
					<li>File picker dialogs may crash the application</li>
					<li>Some native system integrations may be limited</li>
				</ul>

				<div class="bg-blue-500 bg-opacity-10 border border-blue-500 border-opacity-30 rounded-lg p-4">
					<p class="text-sm text-contrast">
						<strong>Workaround:</strong> Use drag & drop to add files instead of file picker dialogs.
					</p>
				</div>

				<p class="text-sm text-secondary">
					These issues are specific to Linux and do not occur on Windows or macOS. We recommend using Windows for the best experience.
				</p>
			</div>

			<div class="px-6 pb-6 flex gap-3">
				<ButtonStyled class="flex-1">
					<button @click="dismissLinuxWarning" class="w-full">
						Don't show again
					</button>
				</ButtonStyled>
				<ButtonStyled type="transparent" class="flex-1">
					<button @click="showLinuxWarning = false" class="w-full">
						Remind me later
					</button>
				</ButtonStyled>
			</div>
		</div>
	</div>

	<div
		v-if="stateInitialized"
		class="app-grid-layout experimental-styles-within relative"
		:class="{ 'disable-advanced-rendering': !themeStore.advancedRendering }"
	>
		<Suspense>
			<AppSettingsModal ref="settingsModal" />
		</Suspense>
		<!-- [AR] Patch: Removed Modrinth login modal -->
		<Suspense>
			<InstanceCreationModal ref="installationModal" />
		</Suspense>
		<div
			class="app-grid-navbar bg-bg-raised flex flex-col p-[0.5rem] pt-0 gap-[0.5rem] w-[--left-bar-width]"
		>
			<NavButton v-tooltip.right="'Home'" to="/">
				<HomeIcon />
			</NavButton>
			<NavButton v-if="themeStore.featureFlags.worlds_tab" v-tooltip.right="'Worlds'" to="/worlds">
				<WorldIcon />
			</NavButton>
			<NavButton
				v-if="themeStore.featureFlags.servers_in_app"
				v-tooltip.right="'Servers'"
				to="/servers/manage"
			>
				<ServerIcon />
			</NavButton>
			<NavButton
				v-tooltip.right="'Discover content'"
				to="/browse/modpack"
				:is-primary="() => route.path.startsWith('/browse') && !route.query.i"
				:is-subpage="(route) => route.path.startsWith('/project') && !route.query.i"
			>
				<CompassIcon />
			</NavButton>
			<!-- [AR] Patch: Skins system removed - hide button -->
			<!-- <NavButton v-tooltip.right="'Skins (Beta)'" to="/skins">
				<ChangeSkinIcon />
			</NavButton> -->
			<NavButton
				v-tooltip.right="'Library'"
				to="/library"
				:is-subpage="
					() =>
						route.path.startsWith('/instance') ||
						((route.path.startsWith('/browse') || route.path.startsWith('/project')) &&
							route.query.i)
				"
			>
				<LibraryIcon />
			</NavButton>
			<div class="h-px w-6 mx-auto my-2 bg-surface-5"></div>
			<suspense>
				<QuickInstanceSwitcher />
			</suspense>
			<NavButton
				v-tooltip.right="'Create new instance'"
				:to="() => installationModal.show()"
				:disabled="offline"
			>
				<PlusIcon />
			</NavButton>
			<div class="flex flex-grow"></div>
			<Transition name="nav-button-animated">
				<div
					v-if="
						availableUpdate &&
						updateToastDismissed &&
						!restarting &&
						(finishedDownloading || metered)
					"
				>
					<NavButton
						v-tooltip.right="
							formatMessage(
								finishedDownloading
									? messages.reloadToUpdate
									: downloadProgress === 0
										? messages.downloadUpdate
										: messages.downloadingUpdate,
								{
									percent: downloadPercent,
								},
							)
						"
						:to="
							finishedDownloading
								? installUpdate
								: downloadProgress > 0 && downloadProgress < 1
									? showUpdateToast
									: downloadAvailableUpdate
						"
					>
						<ProgressSpinner
							v-if="downloadProgress > 0 && downloadProgress < 1"
							class="text-brand"
							:progress="downloadProgress"
						/>
						<RefreshCwIcon v-else-if="finishedDownloading" class="text-brand" />
						<DownloadIcon v-else class="text-brand" />
					</NavButton>
				</div>
			</Transition>
			<template v-if="updateState">
			  <NavButton
			    class="neon-icon pulse"
				v-tooltip.right="formatMessage(commonMessages.settingsLabel)"
				:to="() => settingsModal.show()">
				<SettingsIcon />
			    </NavButton>
      		</template>
      		<template v-else>
      		  <NavButton
				v-tooltip.right="formatMessage(commonMessages.settingsLabel)"
				:to="() => settingsModal.show()">
				<SettingsIcon />
			  </NavButton>
      		</template>
			<OverflowMenu
				v-if="credentials"
				v-tooltip.right="`Modrinth account`"
				class="w-12 h-12 text-primary rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast border-0 cursor-pointer"
				:options="[
					{
						id: 'view-profile',
						action: () => openUrl('https://modrinth.com/user/' + credentials.user.username),
					},
					{
						id: 'sign-out',
						action: () => logOut(),
						color: 'danger',
					},
				]"
				placement="right-end"
			>
				<Avatar :src="credentials.user.avatar_url" alt="" size="32px" circle />
				<template #view-profile>
					<UserIcon />
					<span class="inline-flex items-center gap-1">
						Signed in as
						<span class="inline-flex items-center gap-1 text-contrast font-semibold">
							<Avatar :src="credentials.user.avatar_url" alt="" size="20px" circle />
							{{ credentials.user.username }}
						</span>
					</span>
					<ExternalIcon />
				</template>
				<template #sign-out> <LogOutIcon /> Sign out </template>
			</OverflowMenu>
			<!-- [AR] Patch: Removed Modrinth sign-in button for offline launcher -->
			<!-- Modrinth account login disabled -->
		</div>
		<div data-tauri-drag-region class="app-grid-statusbar bg-bg-raised h-[--top-bar-height] flex">
			<div data-tauri-drag-region class="flex p-3">
				<div data-tauri-drag-region class="flex items-center gap-1 ml-3">
					<button
						class="cursor-pointer p-0 m-0 text-contrast border-none outline-none bg-button-bg rounded-full flex items-center justify-center w-6 h-6 hover:brightness-75 transition-all"
						@click="router.back()"
					>
						<LeftArrowIcon />
					</button>
					<button
						class="cursor-pointer p-0 m-0 text-contrast border-none outline-none bg-button-bg rounded-full flex items-center justify-center w-6 h-6 hover:brightness-75 transition-all"
						@click="router.forward()"
					>
						<RightArrowIcon />
					</button>
				</div>
				<Breadcrumbs class="pt-[2px]" />
			</div>
			<section data-tauri-drag-region class="flex ml-auto items-center">
				<ButtonStyled
					v-if="!forceSidebar && themeStore.toggleSidebar"
					:type="sidebarToggled ? 'standard' : 'transparent'"
					circular
				>
					<button
						class="mr-3 transition-transform"
						:class="{ 'rotate-180': !sidebarToggled }"
						@click="sidebarToggled = !sidebarToggled"
					>
						<RightArrowIcon />
					</button>
				</ButtonStyled>
				<div class="flex mr-3">
					<Suspense>
						<RunningAppBar />
					</Suspense>
				</div>
				<section v-if="!nativeDecorations" class="window-controls" data-tauri-drag-region-exclude>
					<Button class="titlebar-button" icon-only @click="() => getCurrentWindow().minimize()">
						<MinimizeIcon />
					</Button>
					<Button
						class="titlebar-button"
						icon-only
						@click="() => getCurrentWindow().toggleMaximize()"
					>
						<RestoreIcon v-if="isMaximized" />
						<MaximizeIcon v-else />
					</Button>
					<Button class="titlebar-button close" icon-only @click="handleClose">
						<XIcon />
					</Button>
				</section>
			</section>
		</div>
	</div>
	<div
		v-if="stateInitialized"
		class="app-contents experimental-styles-within"
		:class="{
			'sidebar-enabled': sidebarVisible,
			'disable-advanced-rendering': !themeStore.advancedRendering,
		}"
	>
		<div class="app-viewport flex-grow router-view">
			<transition name="popup-survey">
				<div
					v-if="availableSurvey"
					class="w-[400px] z-20 fixed -bottom-12 pb-16 right-[--right-bar-width] mr-4 rounded-t-2xl card-shadow bg-bg-raised border-divider border-[1px] border-solid border-b-0 p-4"
				>
					<h2 class="text-lg font-extrabold mt-0 mb-2">Hey there Modrinth user!</h2>
					<p class="m-0 leading-tight">
						Would you mind answering a few questions about your experience with Modrinth App?
					</p>
					<p class="mt-3 mb-4 leading-tight">
						This feedback will go directly to the Modrinth team and help guide future updates!
					</p>
					<div class="flex gap-2">
						<ButtonStyled color="brand">
							<button @click="openSurvey"><NotepadTextIcon /> Take survey</button>
						</ButtonStyled>
						<ButtonStyled>
							<button @click="dismissSurvey"><XIcon /> No thanks</button>
						</ButtonStyled>
					</div>
				</div>
			</transition>
			<div
				class="loading-indicator-container h-8 fixed z-50"
				:style="{
					top: 'calc(var(--top-bar-height))',
					left: 'calc(var(--left-bar-width))',
					width: 'calc(100% - var(--left-bar-width) - var(--right-bar-width))',
				}"
			>
				<ModrinthLoadingIndicator />
			</div>
			<div
				v-if="themeStore.featureFlags.page_path"
				class="absolute bottom-0 left-0 m-2 bg-tooltip-bg text-tooltip-text font-semibold rounded-full px-2 py-1 text-xs z-50"
			>
				{{ route.fullPath }}
			</div>
			<div
				id="background-teleport-target"
				class="absolute h-full -z-10 rounded-tl-[--radius-xl] overflow-hidden"
				:style="{
					width: 'calc(100% - var(--right-bar-width))',
				}"
			></div>
			<Admonition
				v-if="criticalErrorMessage"
				type="critical"
				:header="criticalErrorMessage.header"
				class="m-6 mb-0"
			>
				<div
					class="markdown-body text-primary"
					v-html="renderString(criticalErrorMessage.body ?? '')"
				></div>
			</Admonition>
			<Admonition
				v-if="authUnreachable"
				type="warning"
				:header="formatMessage(messages.authUnreachableHeader)"
				class="m-6 mb-0"
			>
				{{ formatMessage(messages.authUnreachableBody) }}
			</Admonition>
			<RouterView v-slot="{ Component }">
				<template v-if="Component">
					<Suspense @pending="loading.startLoading()" @resolve="loading.stopLoading()">
						<component :is="Component"></component>
					</Suspense>
				</template>
			</RouterView>
		</div>
		<div
			class="app-sidebar mt-px shrink-0 flex flex-col border-0 border-l-[1px] border-[--brand-gradient-border] border-solid overflow-auto"
			:class="{ 'has-plus': hasPlus }"
		>
			<div
				class="app-sidebar-scrollable flex-grow shrink overflow-y-auto relative"
				:class="{ 'pb-12': !hasPlus }"
			>
				<div id="sidebar-teleport-target" class="sidebar-teleport-content"></div>
				<div class="sidebar-default-content" :class="{ 'sidebar-enabled': sidebarVisible }">
					<div
						class="p-4 pr-1 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid"
					>
						<h3 class="text-base text-primary font-medium m-0">Playing as</h3>
						<suspense>
							<AccountsCard ref="accounts" mode="small" />
						</suspense>
					</div>
					<!-- [AR] Patch: FriendsList disabled -->
					<!--
					<div class="py-4 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid">
						<suspense>
							<FriendsList
								:credentials="credentials"
								:sign-in="() => signIn()"
								:refresh-credentials="fetchCredentials"
							/>
						</suspense>
					</div>
					-->
					<div class="p-4 pr-1 flex flex-col items-center">
						<h3 class="text-base mb-4 text-primary font-medium m-0 text-left w-full">News</h3>
						<div v-if="news && news.length > 0" class="space-y-4 flex flex-col items-center w-full">
							<NewsArticleCard
								v-for="(item, index) in news"
								:key="`news-${index}`"
								:article="item"
							/>
							<ButtonStyled color="brand" size="large">
								<a href="https://github.com/sqrilizz/SqrilizzLauncher/releases" target="_blank" class="my-4">
									<NewspaperIcon /> View all updates
								</a>
							</ButtonStyled>
						</div>
						<div v-else class="w-full flex flex-col items-center justify-center py-12 px-4 text-center">
							<div class="flex items-center justify-center mb-4">
								<NewspaperIcon class="w-12 h-12 text-secondary opacity-40" />
							</div>
							<p class="text-secondary text-sm mb-1 font-medium">No news available</p>
							<p class="text-tertiary text-xs mb-6">Check back later for updates</p>
							<ButtonStyled color="brand" size="small">
								<a href="https://github.com/sqrilizz/SqrilizzLauncher/releases" target="_blank" class="flex items-center gap-2">
									<ExternalIcon /> View releases
								</a>
							</ButtonStyled>
						</div>
					</div>
				</div>
			</div>
		</div>
	</div>
	<URLConfirmModal ref="urlModal" />
	<NotificationPanel has-sidebar />
	<ErrorModal ref="errorModal" />
	<ModInstallModal ref="modInstallModal" />
	<IncompatibilityWarningModal ref="incompatibilityWarningModal" />
	<InstallConfirmModal ref="installConfirmModal" />
</template>

<style lang="scss" scoped>
@import '../../../packages/assets/styles/neon-icon.scss';
@import '../../../packages/assets/styles/neon-text.scss';
.window-controls {
	z-index: 20;
	display: none;
	flex-direction: row;
	align-items: center;

	.titlebar-button {
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		transition: all ease-in-out 0.1s;
		background-color: transparent;
		color: var(--color-base);
		height: 100%;
		width: 3rem;
		position: relative;
		box-shadow: none;

		&:last-child {
			padding-right: 0.75rem;
			width: 3.75rem;
		}

		svg {
			width: 1.25rem;
			height: 1.25rem;
		}

		&::before {
			content: '';
			border-radius: 999999px;
			width: 3rem;
			height: 3rem;
			aspect-ratio: 1 / 1;
			margin-block: auto;
			position: absolute;
			background-color: transparent;
			scale: 0.9;
			transition: all ease-in-out 0.2s;
			z-index: -1;
		}

		&.close {
			&:hover,
			&:active {
				color: var(--color-accent-contrast);

				&::before {
					background-color: var(--color-red);
				}
			}
		}

		&:hover,
		&:active {
			color: var(--color-contrast);

			&::before {
				background-color: var(--color-button-bg);
				scale: 1;
			}
		}
	}
}

.app-grid-layout,
.app-contents {
	--top-bar-height: 3rem;
	--left-bar-width: 4rem;
	--right-bar-width: 300px;
}

.app-grid-layout {
	display: grid;
	grid-template: 'status status' 'nav dummy';
	grid-template-columns: auto 1fr;
	grid-template-rows: auto 1fr;
	position: relative;
	//z-index: 0;
	background-color: var(--color-raised-bg);
	width: 100%;
	height: 100%;
	/* [AR] Removed min-width and min-height to allow window resizing */
}

.app-grid-navbar {
	grid-area: nav;
	position: fixed;
	left: 0;
	top: var(--top-bar-height);
	bottom: 0;
	height: calc(100vh - var(--top-bar-height));
	overflow-y: auto;
	z-index: 100;
}

.app-grid-statusbar {
	grid-area: status;
	position: fixed;
	top: 0;
	left: 0;
	right: 0;
	height: var(--top-bar-height);
	z-index: 101;
}

[data-tauri-drag-region-exclude] {
	-webkit-app-region: no-drag;
}

.app-contents {
	position: absolute;
	z-index: 1;
	left: var(--left-bar-width);
	top: var(--top-bar-height);
	right: 0;
	bottom: 0;
	height: calc(100% - var(--top-bar-height));
	background-color: var(--color-bg);
	border-top-left-radius: var(--radius-xl);

	display: grid;
	grid-template-columns: 1fr 0px;
	// transition: grid-template-columns 0.4s ease-in-out;

	&.sidebar-enabled {
		grid-template-columns: 1fr 300px;
	}
}

.loading-indicator-container {
	border-top-left-radius: var(--radius-xl);
	overflow: hidden;
}

.app-sidebar {
	overflow: visible;
	width: 300px;
	position: relative;
	height: calc(100% - var(--top-bar-height));
	background: var(--brand-gradient-bg);

	--color-button-bg: var(--brand-gradient-button);
	--color-button-bg-hover: var(--brand-gradient-border);
	--color-divider: var(--brand-gradient-border);
	--color-divider-dark: var(--brand-gradient-border);
}

.app-sidebar::before {
	content: '';
	box-shadow: -15px 0 15px -15px rgba(0, 0, 0, 0.1) inset;
	top: 0;
	bottom: 0;
	left: -2rem;
	width: 2rem;
	position: absolute;
	pointer-events: none;
}

.app-viewport {
	flex-grow: 1;
	height: 100%;
	overflow: auto;
	overflow-x: hidden;
}

.app-contents::before {
	z-index: 1;
	content: '';
	position: fixed;
	left: var(--left-bar-width);
	top: var(--top-bar-height);
	right: calc(-1 * var(--left-bar-width));
	bottom: calc(-1 * var(--left-bar-width));
	border-radius: var(--radius-xl);
	box-shadow: 1px 1px 15px rgba(0, 0, 0, 0.1) inset;
	border-color: var(--surface-5);
	border-width: 1px;
	border-style: solid;
	pointer-events: none;
}

.sidebar-teleport-content {
	display: contents;
}

.sidebar-default-content {
	display: none;
}

.sidebar-teleport-content:empty + .sidebar-default-content.sidebar-enabled {
	display: contents;
}

.popup-survey-enter-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.51, 1.08, 0.35, 1.15);
	transform-origin: top center;
}

.popup-survey-leave-active {
	transition:
		opacity 0.25s ease,
		transform 0.25s cubic-bezier(0.68, -0.17, 0.23, 0.11);
	transform-origin: top center;
}

.popup-survey-enter-from,
.popup-survey-leave-to {
	opacity: 0;
	transform: translateY(10rem) scale(0.8) scaleY(1.6);
}

.toast-enter-active {
	transition: opacity 0.25s linear;
}

.toast-enter-from,
.toast-leave-to {
	opacity: 0;
}

@media (prefers-reduced-motion: no-preference) {
	.toast-enter-active,
	.nav-button-animated-enter-active {
		transition: all 0.5s cubic-bezier(0.15, 1.4, 0.64, 0.96);
	}

	.toast-leave-active,
	.nav-button-animated-leave-active {
		transition: all 0.25s ease;
	}

	.toast-enter-from {
		scale: 0.5;
		translate: 0 -10rem;
		opacity: 0;
	}

	.toast-leave-to {
		scale: 0.96;
		translate: 20rem 0;
		opacity: 0;
	}

	.nav-button-animated-enter-active {
		position: relative;
	}

	.nav-button-animated-enter-active::before {
		content: '';
		inset: 0;
		border-radius: 100vw;
		background-color: var(--color-brand-highlight);
		position: absolute;
		animation: pop 0.5s ease-in forwards;
		opacity: 0;
	}

	@keyframes pop {
		0% {
			scale: 0.5;
		}
		50% {
			opacity: 0.5;
		}
		100% {
			scale: 1.5;
		}
	}

	.nav-button-animated-enter-from {
		scale: 0.5;
		translate: -2rem 0;
		opacity: 0;
	}

	.nav-button-animated-leave-to {
		scale: 0.75;
		opacity: 0;
	}

	.fade-enter-active {
		transition: 0.25s ease-in-out;
	}

	.fade-enter-from {
		opacity: 0;
	}
}
</style>
<style>
.mac {
	.app-grid-statusbar {
		padding-left: 5rem;
	}
}

.windows {
	.fake-appbar {
		height: 2.5rem !important;
	}

	.window-controls {
		display: flex !important;
	}

	.info-card {
		right: 8rem;
	}

	.profile-card {
		right: 8rem;
	}
}
</style>
<style src="vue-multiselect/dist/vue-multiselect.css"></style>
