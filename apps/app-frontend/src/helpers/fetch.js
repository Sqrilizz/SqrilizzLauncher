import { getVersion } from '@tauri-apps/api/app'
import { fetch } from '@tauri-apps/plugin-http'

export const useFetch = async (url, item, isSilent) => {
	try {
		const version = await getVersion()
		return await fetch(url, {
			method: 'GET',
			headers: { 'User-Agent': `sqrilizz/launcher/${version}` },
		})
	} catch (err) {
		if (!isSilent) {
			throw err
		} else {
			console.error(err)
		}
	}
}
