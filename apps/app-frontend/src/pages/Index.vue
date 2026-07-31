<script setup lang="ts">
import { CompassIcon, LibraryIcon, PlusIcon } from '@modrinth/assets'
import { injectNotificationManager, useRelativeTime } from '@modrinth/ui'
import type { SearchResult } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, inject, onMounted, onUnmounted, ref } from 'vue'
import { useRoute, useRouter } from 'vue-router'

import defaultHeroBackground from '@/assets/home/sqrilizz-hero.webp'
import RowDisplay from '@/components/RowDisplay.vue'
import Instance from '@/components/ui/Instance.vue'
import { get_search_results } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events'
import { list, run } from '@/helpers/profile.js'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const formatRelativeTime = useRelativeTime()
const router = useRouter()
const route = useRoute()
const openInstanceCreation = inject<(() => void) | null>('openInstanceCreation', null)
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const instances = ref<GameInstance[]>([])
const featuredModpacks = ref<SearchResult[]>([])
const featuredMods = ref<SearchResult[]>([])
const installedModpacksFilter = ref('')
const offline = ref(!navigator.onLine)

const recentInstances = computed(() =>
	instances.value
		.filter((instance) => instance.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

const latestInstance = computed(() => recentInstances.value[0] ?? instances.value[0])
const lastPlayedLabel = computed(() => {
	if (!latestInstance.value?.last_played) return 'Never played'
	return `Last played ${formatRelativeTime(dayjs(latestInstance.value.last_played).toISOString())}`
})
const hasFeaturedProjects = computed(
	() => featuredModpacks.value.length + featuredMods.value.length > 0,
)

const handleOffline = () => {
	offline.value = true
	featuredModpacks.value = []
	featuredMods.value = []
}
const handleOnline = () => {
	offline.value = false
}

async function fetchInstances() {
	instances.value = (await list().catch(handleError)) ?? []
	installedModpacksFilter.value = instances.value
		.filter((instance) => instance.linked_data?.project_id)
		.map((instance) => `NOT"project_id"="${instance.linked_data.project_id}"`)
		.join(' AND ')
}

async function fetchFeaturedModpacks() {
	if (offline.value) return
	try {
		const response = await get_search_results(
			`?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${installedModpacksFilter.value}`,
		)
		featuredModpacks.value = response?.result?.hits ?? []
	} catch (error) {
		featuredModpacks.value = []
		console.warn('Failed to load featured modpacks:', error)
	}
}

async function fetchFeaturedMods() {
	if (offline.value) return
	try {
		const response = await get_search_results(
			'?facets=[["project_type:mod"]]&limit=10&index=follows',
		)
		featuredMods.value = response?.result?.hits ?? []
	} catch (error) {
		featuredMods.value = []
		console.warn('Failed to load featured mods:', error)
	}
}

async function refreshFeaturedProjects() {
	await Promise.all([fetchFeaturedModpacks(), fetchFeaturedMods()])
}

async function refreshHome() {
	await fetchInstances()
	if (offline.value) return

	const refresh = () => void refreshFeaturedProjects()
	if ('requestIdleCallback' in window) {
		window.requestIdleCallback(refresh, { timeout: 1500 })
	} else {
		window.setTimeout(refresh, 250)
	}
}

async function reconnectToRecent() {
	if (!latestInstance.value) {
		await router.push('/library')
		return
	}

	await run(latestInstance.value.path).catch((error) => handleError(error))
}

async function openLibrary() {
	await router.push('/library')
}

async function openCreate() {
	if (openInstanceCreation) {
		openInstanceCreation()
		return
	}
	await router.push('/library')
}

async function openDiscover() {
	await router.push('/browse/modpack')
}

await refreshHome()

onMounted(() => {
	window.addEventListener('offline', handleOffline)
	window.addEventListener('online', handleOnline)
})

const unlistenProfile = await profile_listener(async () => {
	if (!offline.value) await refreshHome()
})

onUnmounted(() => {
	window.removeEventListener('offline', handleOffline)
	window.removeEventListener('online', handleOnline)
	unlistenProfile()
})
</script>

<template>
	<main class="home-page">
		<section class="home-hero premium-panel">
			<div
				class="hero-background"
				:style="{ backgroundImage: `url(${defaultHeroBackground})` }"
				aria-hidden="true"
			/>
			<div class="hero-content">
				<h1>{{ latestInstance ? 'Welcome back.' : 'Your Minecraft, upgraded.' }}</h1>
				<p>
					One clean home for your worlds, modpacks and servers. Pick a profile, press play, and get
					straight into the game.
				</p>
				<div class="hero-actions">
					<button
						class="premium-button premium-button-primary"
						:disabled="Boolean(latestInstance && offline)"
						@click="reconnectToRecent"
					>
						<span class="play-mark">▶</span>
						{{ latestInstance ? `Play ${latestInstance.name}` : 'Open your library' }}
					</button>
					<button class="premium-button premium-button-ghost" @click="openCreate">
						<PlusIcon aria-hidden="true" /> Create instance
					</button>
				</div>
				<div class="hero-meta">
					<span class="status-dot" :class="{ offline }" />
					{{ offline ? 'Offline mode' : 'All systems online' }}
					<span class="meta-separator" />
					{{ lastPlayedLabel }}
				</div>
			</div>
		</section>

		<section class="home-stat-grid">
			<button class="home-stat-card" @click="openLibrary">
				<div class="stat-icon stat-icon-purple">
					<LibraryIcon aria-hidden="true" />
				</div>
				<div>
					<strong>{{ instances.length }}</strong
					><span>Game profiles</span>
				</div>
				<span class="stat-arrow">↗</span>
			</button>
			<button class="home-stat-card" @click="openDiscover">
				<div class="stat-icon stat-icon-cyan">
					<CompassIcon aria-hidden="true" />
				</div>
				<div>
					<strong>{{ featuredModpacks.length + featuredMods.length }}</strong
					><span>Discover picks</span>
				</div>
				<span class="stat-arrow">↗</span>
			</button>
		</section>

		<section class="home-section">
			<div class="section-heading">
				<div>
					<span class="section-kicker">READY TO PLAY</span>
					<h2>Jump back in</h2>
				</div>
				<RouterLink to="/library" class="section-link">View library <span>→</span></RouterLink>
			</div>
			<div v-if="recentInstances.length" class="instance-grid">
				<Instance
					v-for="(instance, index) in recentInstances.slice(0, 4)"
					:key="instance.path"
					:instance="instance"
					:compact="true"
					:first="index === 0"
				/>
			</div>
			<div v-else class="empty-panel premium-panel">
				<div class="empty-icon"><PlusIcon aria-hidden="true" /></div>
				<div>
					<h3>Your next adventure starts here</h3>
					<p>Create an instance or import a modpack to build your library.</p>
				</div>
				<button class="premium-button premium-button-primary" @click="openCreate">
					Create instance
				</button>
			</div>
		</section>

		<section v-if="hasFeaturedProjects" class="home-section discover-section">
			<div class="section-heading">
				<div>
					<span class="section-kicker">CURATED FOR YOU</span>
					<h2>Find your next world</h2>
				</div>
				<RouterLink to="/browse/modpack" class="section-link"
					>Explore all <span>→</span></RouterLink
				>
			</div>
			<RowDisplay
				:instances="[
					{
						label: 'Featured modpacks',
						route: '/browse/modpack',
						instances: featuredModpacks,
						downloaded: false,
					},
					{
						label: 'Popular mods',
						route: '/browse/mod',
						instances: featuredMods,
						downloaded: false,
					},
				]"
				:can-paginate="true"
			/>
		</section>
	</main>
</template>

<style scoped lang="scss">
.home-page {
	max-width: 1500px;
	margin: 0 auto;
	padding: 2.25rem clamp(1.25rem, 4vw, 4rem) 4rem;
	color: var(--sq-text);
}

.home-hero {
	position: relative;
	min-height: 360px;
	overflow: hidden;
	border-radius: 2rem;
	padding: clamp(2rem, 5vw, 4rem);
}

.hero-background {
	position: absolute;
	inset: 0;
	background-position: center;
	background-size: cover;
	opacity: 0.72;
	filter: saturate(1.05) contrast(1.02);
	transition: opacity 180ms ease;
}
.hero-background::after {
	position: absolute;
	inset: 0;
	background:
		linear-gradient(
			90deg,
			rgba(0, 0, 0, 0.62) 0%,
			rgba(0, 0, 0, 0.28) 48%,
			rgba(0, 0, 0, 0.08) 100%
		),
		linear-gradient(0deg, rgba(0, 0, 0, 0.42), transparent 58%);
	content: '';
}

.hero-content {
	position: relative;
	z-index: 2;
	max-width: 650px;
}
.section-kicker {
	color: var(--sq-brand-bright);
	font-size: 0.72rem;
	font-weight: 800;
	letter-spacing: 0.18em;
}
.home-hero h1 {
	max-width: 650px;
	margin: 1rem 0 0.9rem;
	font-size: clamp(2.65rem, 6vw, 5.25rem);
	line-height: 0.96;
	letter-spacing: -0.06em;
}
.home-hero p {
	max-width: 510px;
	margin: 0;
	color: var(--sq-muted);
	font-size: 1.05rem;
	line-height: 1.65;
}
.hero-actions {
	display: flex;
	flex-wrap: wrap;
	gap: 0.8rem;
	margin-top: 2rem;
}
.premium-button {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	gap: 0.6rem;
	min-height: 3rem;
	padding: 0.75rem 1.2rem;
	border: 1px solid transparent;
	border-radius: 0.9rem;
	color: var(--sq-text);
	font-weight: 750;
	cursor: pointer;
	transition:
		transform 180ms ease,
		border-color 180ms ease,
		background 180ms ease;
}
.premium-button:hover:not(:disabled) {
	transform: translateY(-2px);
}
.premium-button:disabled {
	cursor: not-allowed;
	opacity: 0.5;
}
.premium-button svg,
.premium-button i {
	width: 1.1rem;
}
.premium-button-primary {
	background: linear-gradient(135deg, var(--sq-brand), #6d28d9);
	box-shadow: 0 12px 30px rgba(124, 58, 237, 0.32);
}
.premium-button-ghost {
	background: rgba(255, 255, 255, 0.06);
	border-color: var(--sq-border);
}
.play-mark {
	font-size: 0.9rem;
}
.hero-meta {
	display: flex;
	align-items: center;
	gap: 0.55rem;
	margin-top: 1.5rem;
	color: var(--sq-muted);
	font-size: 0.8rem;
}
.status-dot {
	width: 0.55rem;
	height: 0.55rem;
	border-radius: 50%;
	background: var(--sq-success);
	box-shadow: 0 0 14px var(--sq-success);
}
.status-dot.offline {
	background: #f59e0b;
	box-shadow: 0 0 14px #f59e0b;
}
.meta-separator {
	width: 1px;
	height: 1rem;
	margin: 0 0.2rem;
	background: var(--sq-border);
}
.home-stat-grid {
	display: grid;
	grid-template-columns: repeat(2, 1fr);
	gap: 0.9rem;
	margin: 1rem 0 3.5rem;
}
.home-stat-card {
	display: flex;
	align-items: center;
	gap: 0.9rem;
	min-height: 92px;
	padding: 1rem 1.1rem;
	border: 1px solid var(--sq-border);
	border-radius: 1.2rem;
	background: transparent;
	box-shadow: none;
	color: var(--sq-text);
	text-align: left;
	text-decoration: none;
	cursor: pointer;
	transition:
		transform 180ms ease,
		border-color 180ms ease,
		background 180ms ease;
}
.home-stat-card:hover {
	transform: translateY(-3px);
	border-color: var(--sq-border-strong);
	background: rgba(255, 255, 255, 0.025);
}
.stat-icon {
	display: grid;
	place-items: center;
	width: 2.8rem;
	height: 2.8rem;
	border-radius: 0.9rem;
}
.stat-icon svg,
.stat-icon i {
	width: 1.3rem;
	height: 1.3rem;
}
.stat-icon-purple {
	background: rgba(139, 92, 246, 0.16);
	color: var(--sq-brand-bright);
}
.stat-icon-cyan {
	background: rgba(34, 211, 238, 0.12);
	color: var(--sq-cyan);
}
.stat-icon-green {
	background: rgba(52, 211, 153, 0.12);
	color: var(--sq-success);
}
.home-stat-card strong,
.home-stat-card span {
	display: block;
}
.home-stat-card strong {
	font-size: 1.15rem;
}
.home-stat-card div > span {
	margin-top: 0.15rem;
	color: var(--sq-muted);
	font-size: 0.75rem;
}
.stat-arrow {
	margin-left: auto;
	color: var(--sq-muted);
	font-size: 1.2rem;
}
.home-section {
	margin-top: 3rem;
}
.section-heading {
	display: flex;
	align-items: end;
	justify-content: space-between;
	gap: 1rem;
	margin-bottom: 1.1rem;
}
.section-heading h2 {
	margin: 0.35rem 0 0;
	font-size: 1.6rem;
	letter-spacing: -0.03em;
}
.section-link {
	color: var(--sq-muted);
	font-size: 0.82rem;
	font-weight: 700;
	text-decoration: none;
}
.section-link:hover {
	color: var(--sq-text);
}
.section-link span {
	margin-left: 0.25rem;
	color: var(--sq-cyan);
}
.instance-grid {
	display: grid;
	grid-template-columns: repeat(2, minmax(0, 1fr));
	gap: 0.8rem;
}
.empty-panel {
	display: flex;
	align-items: center;
	gap: 1.1rem;
	padding: 1.35rem;
	border-radius: 1.2rem;
}
.empty-panel h3 {
	margin: 0;
	font-size: 1rem;
}
.empty-panel p {
	margin: 0.35rem 0 0;
	color: var(--sq-muted);
	font-size: 0.82rem;
}
.empty-icon {
	display: grid;
	place-items: center;
	width: 3rem;
	height: 3rem;
	border-radius: 1rem;
	background: rgba(139, 92, 246, 0.15);
	color: var(--sq-brand-bright);
}
.empty-icon svg,
.empty-icon i {
	font-size: 1.25rem;
}
.empty-panel .premium-button {
	margin-left: auto;
}
.discover-section :deep(.row-display),
.discover-section :deep(.grid-display) {
	background: transparent;
}
@media (max-width: 1050px) {
	.home-stat-grid {
		grid-template-columns: 1fr;
	}
}

@media (max-width: 760px) {
	.home-page {
		padding: 1rem 1rem 3rem;
	}
	.home-hero {
		min-height: 460px;
		padding: 2rem 1.5rem;
	}
	.instance-grid {
		grid-template-columns: 1fr;
	}
	.section-heading {
		align-items: start;
		flex-direction: column;
	}
	.empty-panel {
		align-items: flex-start;
		flex-wrap: wrap;
	}
	.empty-panel .premium-button {
		margin-left: 4.1rem;
	}
}
</style>
