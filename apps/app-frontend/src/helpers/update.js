import { getVersion } from '@tauri-apps/api/app'
import { ref } from 'vue'

import { getOS, initUpdateLauncher } from '@/helpers/utils.js'

export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)

const currentOS = ref('')
const api = `https://github.com/sqrilizz/SqrilizzLauncher/releases/latest`

const systems = ['macos', 'windows', 'linux']
const macosExtensions = ['.dmg', '.pkg', '.app']
const windowsExtensions = ['.exe', '.msi']

const blacklistBeginPrefixes = [
	`dev`,
	`nightly`,
	`dirty`,
	`dirty-dev`,
	`dirty-nightly`,
	`dirty_dev`,
	`dirty_nightly`,
] // This is blacklisted builds for download. For example, file.startsWith('dev') is not allowed.

export async function getRemote(isDownloadState) {
	var releaseTag = null;
	var releaseTitle = null;
	var result = false;
	currentOS.value = await getOS();
	try {
		const response = await fetch(api);
		if (!response.ok) {
			throw new Error(response.status);
		}
		const remoteData = await response.json();
		releaseTag = document.getElementById('releaseTag');
		releaseTitle = document.getElementById('releaseTitle');
		if (releaseTag && releaseTitle) {
			releaseTag.textContent = remoteData.tag_name;
			releaseTitle.textContent = remoteData.name;
		}
		if (systems.includes(currentOS.value.toLowerCase())) {
			const localVersion = await getVersion();
			const isUpdateAvailable = !remoteData.tag_name.includes(localVersion);
			updateState.value = isUpdateAvailable;
			allowState.value = isUpdateAvailable;
		} else {
			updateState.value = false;
			allowState.value = false;
		}
		if (isDownloadState) {
			try {
				installState.value = true;
				const builds = remoteData.assets;
				const fileName = getInstaller(getExtension(), builds);
				result = fileName ? await initUpdateLauncher(fileName[1], fileName[0], currentOS.value, true) : false;
				installState.value = false;
			} catch (err) {
				installState.value = false;
			}
		}
		console.log('Update available state is', updateState.value);
		console.log('Remote version is', remoteData.tag_name);
		console.log('Remote title is', remoteData.name);
		console.log('Local version is', await getVersion());
		console.log('Operating System is', currentOS.value);
		return result;
	} catch (error) {
		console.error("Failed to fetch remote releases:", error);
		if (!releaseTag) {
			updateState.value = false;
			allowState.value = false;
			installState.value = false;
		}
	}
}

function getInstaller(osExtension, builds) {
	console.log(osExtension, builds)
	for (const build of builds) {
		if (blacklistBeginPrefixes.some(prefix => build.name.startsWith(prefix))) {
			continue;
		}
		if (osExtension.some(ext => build.name.endsWith(ext))) {
			console.log(build.name, build.browser_download_url);
			return [build.name, build.browser_download_url];
		}
	}
	return null;
}

function getExtension() {
	return systems.find(osName => osName === currentOS.value.toLowerCase())?.endsWith('macos')
		? macosExtensions
		: windowsExtensions;
}
