<script setup lang="ts">
import {
	CoffeeIcon,
	DownloadIcon,
	GameIcon,
	GaugeIcon,
	PaintbrushIcon,
	ReportIcon,
	SettingsIcon,
	ShieldIcon,
	SpinnerIcon,
} from '@modrinth/assets'
import { Button, ProgressBar, TabbedModal } from '@modrinth/ui'
import { getVersion } from '@tauri-apps/api/app'
import { platform as getOsPlatform, version as getOsVersion } from '@tauri-apps/plugin-os'
import { defineMessage, defineMessages, useVIntl } from '@vintl/vintl'
import { computed, ref, watch } from 'vue'

import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import AppearanceSettings from '@/components/ui/settings/AppearanceSettings.vue'
import DefaultInstanceSettings from '@/components/ui/settings/DefaultInstanceSettings.vue'
import FeatureFlagSettings from '@/components/ui/settings/FeatureFlagSettings.vue'
import JavaSettings from '@/components/ui/settings/JavaSettings.vue'
import PrivacySettings from '@/components/ui/settings/PrivacySettings.vue'
import ResourceManagementSettings from '@/components/ui/settings/ResourceManagementSettings.vue'
import { get, set } from '@/helpers/settings.ts'
import { getRemote, installState, latestRelease, updateState } from '@/helpers/update.js'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useTheming } from '@/store/state'

const updateModalView = ref(null)
const updateRequestFailView = ref(null)

const initDownload = async () => {
	updateModalView.value?.hide()
	const result = await getRemote(true)
	if (!result) {
		updateRequestFailView.value?.show()
	}
}

const showUpdateModal = async () => {
	if (await getRemote(false)) updateModalView.value?.show()
}

const themeStore = useTheming()

const { formatMessage } = useVIntl()

const devModeCounter = ref(0)

const developerModeEnabled = defineMessage({
	id: 'app.settings.developer-mode-enabled',
	defaultMessage: 'Developer mode enabled.',
})

const tabs = [
	{
		name: defineMessage({
			id: 'app.settings.tabs.appearance',
			defaultMessage: 'Appearance',
		}),
		icon: PaintbrushIcon,
		content: AppearanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.privacy',
			defaultMessage: 'Privacy',
		}),
		icon: ShieldIcon,
		content: PrivacySettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.java-installations',
			defaultMessage: 'Java installations',
		}),
		icon: CoffeeIcon,
		content: JavaSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.default-instance-options',
			defaultMessage: 'Default instance options',
		}),
		icon: GameIcon,
		content: DefaultInstanceSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.resource-management',
			defaultMessage: 'Resource management',
		}),
		icon: GaugeIcon,
		content: ResourceManagementSettings,
	},
	{
		name: defineMessage({
			id: 'app.settings.tabs.feature-flags',
			defaultMessage: 'Feature flags',
		}),
		icon: ReportIcon,
		content: FeatureFlagSettings,
		developerOnly: true,
	},
]

const modal = ref()

function show() {
	modal.value?.show()
}

const isOpen = computed(() => modal.value?.isOpen)

defineExpose({ show, isOpen })

const { progress, version: downloadingVersion } = injectAppUpdateDownloadProgress()

const version = await getVersion()
const osPlatform = getOsPlatform()
const osVersion = getOsVersion()
const settings = ref(await get())

watch(
	settings,
	async () => {
		await set(settings.value)
	},
	{ deep: true },
)

function devModeCount() {
	devModeCounter.value++
	if (devModeCounter.value > 5) {
		themeStore.devMode = !themeStore.devMode
		settings.value.developer_mode = !!themeStore.devMode
		devModeCounter.value = 0

		if (!themeStore.devMode && modal.value && tabs[modal.value.selectedTab].developerOnly) {
			modal.value.setTab(0)
		}
	}
}

const messages = defineMessages({
	downloading: {
		id: 'app.settings.downloading',
		defaultMessage: 'Downloading v{version}',
	},
})
</script>
<template>
	<ModalWrapper ref="modal">
		<template #title>
			<span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
				<SettingsIcon /> Settings
			</span>
		</template>

		<TabbedModal :tabs="tabs">
			<template #footer>
				<div class="settings-footer-content">
					<div class="mb-3">
						<template v-if="progress > 0 && progress < 1">
							<p class="m-0 mb-2">
								{{ formatMessage(messages.downloading, { version: downloadingVersion }) }}
							</p>
							<ProgressBar :progress="progress" />
						</template>
					</div>
					<p v-if="themeStore.devMode" class="text-brand font-semibold m-0 mb-2">
						{{ formatMessage(developerModeEnabled) }}
					</p>
					<div class="settings-app-footer flex items-center gap-3">
						<button
							class="settings-brand-mark p-0 m-0 border-none cursor-pointer button-animation"
							:class="{
								'text-brand': themeStore.devMode,
								'text-secondary': !themeStore.devMode,
							}"
							@click="devModeCount"
						>
							<span aria-hidden="true">✦</span>
						</button>
						<div class="settings-brand-copy">
							<p class="settings-brand-name m-0">Sqrilizz Launcher</p>
							<p class="m-0 text-secondary">
								v{{ version }} ·
								<span v-if="osPlatform === 'macos'">macOS</span>
								<span v-else class="capitalize">{{ osPlatform }}</span>
								{{ osVersion }}
							</p>
						</div>
						<div
							v-if="updateState"
							class="settings-update-action w-8 h-8 cursor-pointer hover:brightness-110 neon-icon pulse"
						>
							<template v-if="installState">
								<SpinnerIcon
									v-tooltip.bottom="'Installing in process...'"
									class="size-6 animate-spin"
								/>
							</template>
							<template v-else>
								<DownloadIcon
									v-tooltip.bottom="'View update info'"
									class="size-6"
									@click="!installState && showUpdateModal()"
								/>
							</template>
						</div>
					</div>
				</div>
			</template>
		</TabbedModal>
		<!-- [SL] Feature -->
		<ModalWrapper
			ref="updateModalView"
			:has-to-type="false"
			header="Request to update the Sqrilizz Launcher"
		>
			<div class="space-y-4">
				<div class="space-y-2">
					<strong>The new version of the Sqrilizz Launcher is available!</strong>
					<p>Your version is outdated. We recommend that you update to the latest version.</p>
					<br />
					<br />
					<p><strong>⚠️ Please, read this notice before initialize update process</strong></p>
					<p>
						Before updating, make sure that you have saved and closed all running instances and made
						a backup copy of the launcher data such as
						<code>%appdata%\Roaming\SqrilizzLauncher</code> on Windows or
						<code>~/Library/Application Support/SqrilizzLauncher</code> on macOS. Remember that the
						authors of the product are not responsible for the breakdown of your files, so you
						should always make back up copies of them and keep them in a safe place.
					</p>
				</div>
				<div class="text-sm text-secondary space-y-1">
					<p>
						<strong>☁️ Latest release tag:</strong>
						<span class="neon-text">{{ latestRelease.tag || 'Unknown' }}</span>
						<br />
						<strong>☁️ Latest release title:</strong>
						<span class="neon-text">{{ latestRelease.title || 'Unknown' }}</span>
						<br />
						<strong>💾 Installed & Running version:</strong>
						<span class="neon-text">v{{ version }}</span>
					</p>
				</div>
				<a
					class="neon-text"
					href="https://github.com/sqrilizz/SqrilizzLauncher"
					target="_blank"
					rel="noopener noreferrer"
				>
					Checkout our git repository
				</a>
				<div class="absolute bottom-4 right-4 flex items-center gap-4 neon-button neon">
					<Button class="bordered" @click="updateModalView.hide()">Cancel</Button>
					<Button class="bordered" @click="initDownload()">Download file</Button>
				</div>
			</div>
		</ModalWrapper>
		<ModalWrapper
			ref="updateRequestFailView"
			:has-to-type="false"
			header="Failed to request a file from the server :("
		>
			<div class="space-y-4">
				<div class="space-y-2">
					<p><strong>Error occurred</strong></p>
					<p>Unfortunately, the program was unable to download the file from our servers.</p>
					<p>
						Please try downloading it yourself from
						<a
							class="neon-text"
							href="https://github.com/sqrilizz/SqrilizzLauncher"
							target="_blank"
							rel="noopener noreferrer"
							>GitHub</a
						>
						if there are any updates available.
					</p>
				</div>

				<div class="text-sm text-secondary">
					<p>
						<strong>Local Sqrilizz:</strong>
						<span class="neon-text">v{{ version }}</span>
					</p>
				</div>

				<div class="absolute bottom-4 right-4 flex items-center gap-4 neon-button neon">
					<Button class="bordered" @click="updateRequestFailView.hide()">Close</Button>
				</div>
			</div>
		</ModalWrapper>
	</ModalWrapper>
</template>

<style lang="scss" scoped>
@import '../../../../../../packages/assets/styles/neon-icon.scss';
@import '../../../../../../packages/assets/styles/neon-button.scss';
@import '../../../../../../packages/assets/styles/neon-text.scss';

code {
	background: linear-gradient(90deg, #005eff, #00cfff);
	background-clip: text;
	-webkit-background-clip: text;
	color: transparent;
}

.settings-footer-content {
	color: var(--color-secondary);
	font-size: 0.75rem;
}

.settings-footer-content > .mb-3 {
	margin-bottom: 0.7rem;
}

.settings-app-footer {
	display: grid;
	grid-template-columns: auto minmax(0, 1fr) auto;
	align-items: center;
	gap: 0.7rem;
	min-height: 4rem;
	padding: 0.7rem;
	border: 1px solid color-mix(in srgb, var(--color-divider) 90%, transparent);
	border-radius: 0.95rem;
	background: transparent;
	box-shadow: none;
}

.settings-brand-mark {
	display: grid;
	place-items: center;
	width: 2.35rem;
	height: 2.35rem;
	border: 1px solid color-mix(in srgb, var(--color-brand) 40%, var(--color-divider));
	border-radius: 0.75rem;
	background: linear-gradient(
		145deg,
		color-mix(in srgb, var(--color-brand) 28%, var(--color-bg)),
		color-mix(in srgb, var(--color-brand) 9%, var(--color-bg))
	);
	color: var(--color-brand);
	font-size: 1.2rem;
	line-height: 1;
	box-shadow: 0 0.45rem 1rem color-mix(in srgb, var(--color-brand) 12%, transparent);
}

.settings-brand-name {
	color: var(--color-contrast);
	font-size: 0.8rem;
	font-weight: 850;
	letter-spacing: -0.01em;
}

.settings-brand-copy p:last-child {
	margin-top: 0.25rem;
	font-size: 0.68rem;
	line-height: 1.3;
	white-space: nowrap;
	overflow: hidden;
	text-overflow: ellipsis;
}

.settings-update-action {
	display: grid;
	place-items: center;
	border: 1px solid color-mix(in srgb, var(--color-brand) 28%, var(--color-divider));
	border-radius: 0.65rem;
	background: color-mix(in srgb, var(--color-brand) 12%, var(--color-button-bg));
	color: var(--color-brand);
}
</style>
