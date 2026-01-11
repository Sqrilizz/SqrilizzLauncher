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
	SqrilizzLogo
} from '@modrinth/assets'
import { ProgressBar, TabbedModal } from '@modrinth/ui'
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

// [AR] Imports
import { getRemote, installState, updateState } from '@/helpers/update.js'
import { injectAppUpdateDownloadProgress } from '@/providers/download-progress.ts'
import { useTheming } from '@/store/state'

const updateModalView = ref(null)
const updateRequestFailView = ref(null)

const initUpdateModal = async () => {
  updateModalView.value.show()
}

const initDownload = async () => {
  updateModalView.value.hide()
  const result = await getRemote(true);
  if (!result) {
    updateRequestFailView.value.show()
  }
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
	modal.value.show()
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

		if (!themeStore.devMode && tabs[modal.value.selectedTab].developerOnly) {
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

		<TabbedModal :tabs="tabs.filter((t) => !t.developerOnly || themeStore.devMode)">
			<template #footer>
				<div class="mt-auto text-secondary text-sm">
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
					<div class="flex items-center gap-3">
						<button
							class="p-0 m-0 bg-transparent border-none cursor-pointer button-animation"
							:class="{
								'text-brand': themeStore.devMode,
								'text-secondary': !themeStore.devMode,
							}"
							@click="devModeCount"
						>
							<SqrilizzLogo class="w-6 h-6" />
						</button>
						<div>
							<p class="m-0">Sqrilizz Launcher {{ version }}</p>
							<p class="m-0">
								<span v-if="osPlatform === 'macos'">macOS</span>
								<span v-else class="capitalize">{{ osPlatform }}</span>
								{{ osVersion }}
							</p>
						</div>
						<div v-if="updateState" class="w-8 h-8 cursor-pointer hover:brightness-75 neon-icon pulse">
            			  <template v-if="installState">
            			    <SpinnerIcon class="size-6 animate-spin" v-tooltip.bottom="'Installing in process...'" />
            			  </template>
            			  <template v-else>
            			    <DownloadIcon class="size-6" v-tooltip.bottom="'View update info'" @click="!installState && (initUpdateModal(), getRemote(false))" />
            			  </template>
            			</div>
					</div>
				</div>
			</template>
		</TabbedModal>
		<!-- [SL] Feature -->
    	<ModalWrapper ref="updateModalView" :has-to-type="false" header="Request to update the Sqrilizz Launcher">
    	  <div class="space-y-4">
    	    <div class="space-y-2">
    	      <strong>The new version of the Sqrilizz Launcher is available!</strong>
    	      <p>Your version is outdated. We recommend that you update to the latest version.</p>
			  <br/>
			  <br/>
    	      <p><strong>⚠️ Please, read this notice before initialize update process</strong></p>
    	      <p>
    	        Before updating, make sure that you have saved and closed all running instances and made a backup copy of the launcher data such as
				<code>%appdata%\Roaming\SqrilizzLauncher</code> on Windows or <code>~/Library/Application Support/SqrilizzLauncher</code> on macOS.
				Remember that the authors of the product are not responsible for the breakdown of
    	        your files, so you should always make back up copies of them and keep them in a safe place.
    	      </p>
    	    </div>
    	    <div class="text-sm text-secondary space-y-1">
    	      <p>
    	        <strong>☁️ Latest release tag:</strong>
    	        <span id="releaseTag" class="neon-text"></span>
		        <br/>
				<strong>☁️ Latest release title:</strong>
				<span id="releaseTitle" class="neon-text"></span>
				<br/>
    	        <strong>💾 Installed & Running version:</strong>
    	        <span class="neon-text">v{{ version }}</span>
    	      </p>
    	    </div>
			  <a class="neon-text" href="https://github.com/sqrilizz/SqrilizzLauncher" target="_blank"
    	        rel="noopener noreferrer">
				Checkout our git repository
			  </a>
    	    <div class="absolute bottom-4 right-4 flex items-center gap-4 neon-button neon">
    	      <Button class="bordered" @click="updateModalView.hide()">Cancel</Button>
    	      <Button class="bordered" @click="initDownload()">Download file</Button>
    	    </div>
    	  </div>
    	</ModalWrapper>
    	<ModalWrapper ref="updateRequestFailView" :has-to-type="false" header="Failed to request a file from the server :(">
    	  <div class="space-y-4">
    	    <div class="space-y-2">
    	      <p><strong>Error occurred</strong></p>
    	      <p>Unfortunately, the program was unable to download the file from our servers.</p>
    	      <p>
    	        Please try downloading it yourself from
    	        <a class="neon-text" href="https://github.com/sqrilizz/SqrilizzLauncher" target="_blank" rel="noopener noreferrer">GitHub</a>
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
</style>
