<script setup lang="ts">
import { injectNotificationManager } from '@modrinth/ui'
import type { SearchResult } from '@modrinth/utils'
import dayjs from 'dayjs'
import { computed, onUnmounted, ref } from 'vue'
import { useRoute } from 'vue-router'

import RowDisplay from '@/components/RowDisplay.vue'
import RecentWorldsList from '@/components/ui/world/RecentWorldsList.vue'
import { get_search_results } from '@/helpers/cache.js'
import { profile_listener } from '@/helpers/events'
import { list } from '@/helpers/profile.js'
import type { GameInstance } from '@/helpers/types'
import { useBreadcrumbs } from '@/store/breadcrumbs'

const { handleError } = injectNotificationManager()
const route = useRoute()
const breadcrumbs = useBreadcrumbs()

breadcrumbs.setRootContext({ name: 'Home', link: route.path })

const instances = ref<GameInstance[]>([])

const featuredModpacks = ref<SearchResult[]>([])
const featuredMods = ref<SearchResult[]>([])
const installedModpacksFilter = ref('')

const recentInstances = computed(() =>
	instances.value
		.filter((x) => x.last_played)
		.slice()
		.sort((a, b) => dayjs(b.last_played).diff(dayjs(a.last_played))),
)

const hasFeaturedProjects = computed(
	() => (featuredModpacks.value?.length ?? 0) + (featuredMods.value?.length ?? 0) > 0,
)

const offline = ref<boolean>(!navigator.onLine)
window.addEventListener('offline', () => {
	offline.value = true
})
window.addEventListener('online', () => {
	offline.value = false
})

async function fetchInstances() {
	instances.value = await list().catch(handleError)

	const filters = []
	for (const instance of instances.value) {
		if (instance.linked_data && instance.linked_data.project_id) {
			filters.push(`NOT"project_id"="${instance.linked_data.project_id}"`)
		}
	}
	installedModpacksFilter.value = filters.join(' AND ')
}

async function fetchFeaturedModpacks() {
	const response = await get_search_results(
		`?facets=[["project_type:modpack"]]&limit=10&index=follows&filters=${installedModpacksFilter.value}`,
	)

	if (response) {
		featuredModpacks.value = response.result.hits
	} else {
		featuredModpacks.value = []
	}
}

async function fetchFeaturedMods() {
	const response = await get_search_results('?facets=[["project_type:mod"]]&limit=10&index=follows')

	if (response) {
		featuredMods.value = response.result.hits
	} else {
		featuredModpacks.value = []
	}
}

async function refreshFeaturedProjects() {
	await Promise.all([fetchFeaturedModpacks(), fetchFeaturedMods()])
}

await fetchInstances()
await refreshFeaturedProjects()

const unlistenProfile = await profile_listener(
	async (e: { event: string; profile_path_id: string }) => {
		await fetchInstances()

		if (e.event === 'added' || e.event === 'created' || e.event === 'removed') {
			await refreshFeaturedProjects()
		}
	},
)

onUnmounted(() => {
	unlistenProfile()
})

async function reconnectToRecent() {
	if (recentInstances.value.length > 0) {
		const mostRecent = recentInstances.value[0]
		await import('@/helpers/profile.js').then(async ({ run }) => {
			try {
				await run(mostRecent.path)
			} catch (error) {
				handleError(error)
			}
		})
	}
}
</script>

<template>
	<div class="p-6 flex flex-col gap-4">
		<h1 v-if="recentInstances?.length > 0" class="m-0 text-2xl font-extrabold">Welcome back!</h1>
		<h1 v-else class="m-0 text-2xl font-extrabold">Welcome to Sqrilizz Launcher!</h1>

		<!-- Recent Worlds/Instances List -->
		<RecentWorldsList :recent-instances="recentInstances" />

		<RowDisplay
			v-if="hasFeaturedProjects"
			:instances="[
				{
					label: 'Discover a modpack',
					route: '/browse/modpack',
					instances: featuredModpacks,
					downloaded: false,
				},
				{
					label: 'Discover mods',
					route: '/browse/mod',
					instances: featuredMods,
					downloaded: false,
				},
			]"
			:can-paginate="true"
		/>

		<!-- Social Links - компактно внизу -->
		<div class="social-links-compact mt-auto pt-4 border-t border-divider">
			<div class="flex gap-3 justify-center">
				<a
					href="https://t.me/sqrilizz"
					target="_blank"
					class="social-link-compact"
					title="Telegram"
				>
					<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
						<path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm4.64 6.8c-.15 1.58-.8 5.42-1.13 7.19-.14.75-.42 1-.68 1.03-.58.05-1.02-.38-1.58-.75-.88-.58-1.38-.94-2.23-1.5-.99-.65-.35-1.01.22-1.59.15-.15 2.71-2.48 2.76-2.69a.2.2 0 00-.05-.18c-.06-.05-.14-.03-.21-.02-.09.02-1.49.95-4.22 2.79-.4.27-.76.41-1.08.4-.36-.01-1.04-.2-1.55-.37-.63-.2-1.12-.31-1.08-.66.02-.18.27-.36.74-.55 2.92-1.27 4.86-2.11 5.83-2.51 2.78-1.16 3.35-1.36 3.73-1.36.08 0 .27.02.39.12.1.08.13.19.14.27-.01.06.01.24 0 .38z"/>
					</svg>
				</a>
				<a
					href="https://sqrilizz.xyz"
					target="_blank"
					class="social-link-compact"
					title="Website"
				>
					<svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
						<circle cx="12" cy="12" r="10"/>
						<path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/>
					</svg>
				</a>
				<a
					href="https://github.com/sqrilizz/SqrilizzLauncher"
					target="_blank"
					class="social-link-compact"
					title="GitHub"
				>
					<svg class="w-5 h-5" viewBox="0 0 24 24" fill="currentColor">
						<path d="M12 2A10 10 0 002 12c0 4.42 2.87 8.17 6.84 9.5.5.08.66-.23.66-.5v-1.69c-2.77.6-3.36-1.34-3.36-1.34-.46-1.16-1.11-1.47-1.11-1.47-.91-.62.07-.6.07-.6 1 .07 1.53 1.03 1.53 1.03.87 1.52 2.34 1.07 2.91.83.09-.65.35-1.09.63-1.34-2.22-.25-4.55-1.11-4.55-4.92 0-1.11.38-2 1.03-2.71-.1-.25-.45-1.29.1-2.64 0 0 .84-.27 2.75 1.02.79-.22 1.65-.33 2.5-.33.85 0 1.71.11 2.5.33 1.91-1.29 2.75-1.02 2.75-1.02.55 1.35.2 2.39.1 2.64.65.71 1.03 1.6 1.03 2.71 0 3.82-2.34 4.66-4.57 4.91.36.31.69.92.69 1.85V21c0 .27.16.59.67.5C19.14 20.16 22 16.42 22 12A10 10 0 0012 2z"/>
					</svg>
				</a>
			</div>
		</div>
	</div>
</template>

<style scoped>
.social-link-compact {
	display: flex;
	align-items: center;
	justify-content: center;
	width: 2.5rem;
	height: 2.5rem;
	border-radius: 0.5rem;
	color: var(--color-secondary);
	background-color: var(--color-bg-raised);
	transition: all 0.2s ease;
	text-decoration: none;
}

.social-link-compact:hover {
	color: var(--color-brand);
	background-color: var(--color-surface-3);
	transform: translateY(-2px);
}

.social-links-compact {
	opacity: 0.7;
	transition: opacity 0.2s ease;
}

.social-links-compact:hover {
	opacity: 1;
}
</style>
