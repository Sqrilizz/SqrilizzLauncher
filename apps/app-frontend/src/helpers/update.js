import { getVersion } from '@tauri-apps/api/app'
import { ref } from 'vue'

import { getOS, initUpdateLauncher } from '@/helpers/utils.js'

export const allowState = ref(false)
export const installState = ref(false)
export const updateState = ref(false)
export const latestRelease = ref({ tag: '', title: '' })

const currentOS = ref('')
const api = 'https://api.github.com/repos/sqrilizz/SqrilizzLauncher/releases/latest'

const installerExtensions = {
	windows: ['.msi', '.exe'],
	linux: ['.appimage', '.deb', '.rpm'],
	macos: ['.dmg', '.pkg', '.app'],
}

const blacklistBeginPrefixes = [
	'dev',
	'nightly',
	'dirty',
	'dirty-dev',
	'dirty-nightly',
	'dirty_dev',
	'dirty_nightly',
]

export async function getRemote(isDownloadState = false) {
	try {
		currentOS.value = String(await getOS()).toLowerCase()
		const response = await fetch(api, {
			headers: { Accept: 'application/vnd.github+json' },
		})
		if (!response.ok) throw new Error(`GitHub release request failed with ${response.status}`)

		const remoteData = await response.json()
		const localVersion = normalizeVersion(await getVersion())
		const remoteVersion = normalizeVersion(remoteData.tag_name)
		const isUpdateAvailable = compareVersions(remoteVersion, localVersion) > 0

		updateState.value = isUpdateAvailable
		allowState.value = isUpdateAvailable
		setReleaseMetadata(remoteData)

		if (!isDownloadState) return isUpdateAvailable
		if (!isUpdateAvailable) return false

		const installer = getInstaller(
			installerExtensions[currentOS.value] ?? [],
			remoteData.assets ?? [],
		)
		if (!installer) return false

		installState.value = true
		await initUpdateLauncher(installer.url, installer.name, currentOS.value, true)
		return true
	} catch (error) {
		console.error('Failed to check or download launcher update:', error)
		return false
	} finally {
		installState.value = false
	}
}

function normalizeVersion(version) {
	return String(version ?? '')
		.trim()
		.replace(/^v/i, '')
}

function setReleaseMetadata(release) {
	latestRelease.value = {
		tag: release.tag_name ?? 'Unknown',
		title: release.name ?? release.tag_name ?? 'Unknown',
	}
}

function getInstaller(extensions, builds) {
	return (
		builds
			.filter((build) => build?.name && build?.browser_download_url)
			.filter((build) =>
				extensions.some((extension) => build.name.toLowerCase().endsWith(extension)),
			)
			.filter(
				(build) =>
					!blacklistBeginPrefixes.some((prefix) => build.name.toLowerCase().startsWith(prefix)),
			)
			.sort((a, b) => extensionRank(a.name, extensions) - extensionRank(b.name, extensions))[0] ??
		null
	)
}

function extensionRank(filename, extensions) {
	const normalized = filename.toLowerCase()
	const rank = extensions.findIndex((extension) => normalized.endsWith(extension))
	return rank === -1 ? Number.MAX_SAFE_INTEGER : rank
}

function compareVersions(left, right) {
	const leftVersion = parseVersion(left)
	const rightVersion = parseVersion(right)
	if (!leftVersion || !rightVersion) return 0

	for (let index = 0; index < 3; index++) {
		if (leftVersion.numbers[index] !== rightVersion.numbers[index]) {
			return leftVersion.numbers[index] > rightVersion.numbers[index] ? 1 : -1
		}
	}

	if (!leftVersion.prerelease && rightVersion.prerelease) return 1
	if (leftVersion.prerelease && !rightVersion.prerelease) return -1
	if (leftVersion.prerelease === rightVersion.prerelease) return 0

	const leftParts = leftVersion.prerelease.split('.')
	const rightParts = rightVersion.prerelease.split('.')
	const length = Math.max(leftParts.length, rightParts.length)
	for (let index = 0; index < length; index++) {
		const leftPart = leftParts[index]
		const rightPart = rightParts[index]
		if (leftPart === undefined) return -1
		if (rightPart === undefined) return 1
		if (leftPart === rightPart) continue

		const leftNumber = /^\d+$/.test(leftPart) ? Number(leftPart) : null
		const rightNumber = /^\d+$/.test(rightPart) ? Number(rightPart) : null
		if (leftNumber !== null && rightNumber !== null) return leftNumber > rightNumber ? 1 : -1
		if (leftNumber !== null) return -1
		if (rightNumber !== null) return 1
		return leftPart > rightPart ? 1 : -1
	}

	return 0
}

function parseVersion(version) {
	const match = normalizeVersion(version).match(
		/^(\d+)(?:\.(\d+))?(?:\.(\d+))?(?:-([0-9A-Za-z.-]+))?$/,
	)
	if (!match) return null

	return {
		numbers: [Number(match[1]), Number(match[2] ?? 0), Number(match[3] ?? 0)],
		prerelease: match[4] ?? '',
	}
}
