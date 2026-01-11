/**
 * Анонимная аналитика для Sqrilizz Launcher
 * Полностью анонимно - не собирает личные данные
 * Только подсчет активных пользователей
 */

import { getVersion } from '@tauri-apps/api/app'

const HEARTBEAT_INTERVAL = 24 * 60 * 60 * 1000 // 24 часа
const STORAGE_KEY_ID = 'sqrilizz_anonymous_id'
const STORAGE_KEY_LAST_HEARTBEAT = 'sqrilizz_last_heartbeat'

/**
 * Генерирует анонимный ID пользователя
 * ID сохраняется локально и используется только для подсчета уникальных пользователей
 */
function getOrCreateAnonymousId() {
	let anonymousId = localStorage.getItem(STORAGE_KEY_ID)

	if (!anonymousId) {
		// Генерируем простой анонимный ID
		anonymousId =
			'user_' +
			Math.random().toString(36).substring(2, 15) +
			Math.random().toString(36).substring(2, 15)
		localStorage.setItem(STORAGE_KEY_ID, anonymousId)
	}

	return anonymousId
}

/**
 * Проверяет нужно ли отправлять heartbeat
 */
function shouldSendHeartbeat() {
	const lastHeartbeat = localStorage.getItem(STORAGE_KEY_LAST_HEARTBEAT)

	if (!lastHeartbeat) {
		return true
	}

	const now = Date.now()
	const lastTime = parseInt(lastHeartbeat, 10)

	return now - lastTime > HEARTBEAT_INTERVAL
}

/**
 * Отправляет анонимный heartbeat
 * Использует GitHub API для простого подсчета
 */
export async function sendAnonymousHeartbeat() {
	try {
		if (!shouldSendHeartbeat()) {
			console.debug('[Analytics] Heartbeat not needed yet')
			return
		}

		const anonymousId = getOrCreateAnonymousId()
		const version = await getVersion()

		// Используем GitHub API для подсчета просмотров релизов
		// Это полностью анонимно и не требует дополнительного сервера
		const response = await fetch(
			'https://api.github.com/repos/sqrilizz/SqrilizzLauncher/releases/latest',
			{
				method: 'GET',
				headers: {
					'User-Agent': `SqrilizzLauncher/${version}`,
					Accept: 'application/vnd.github.v3+json',
				},
			},
		)

		if (response.ok) {
			localStorage.setItem(STORAGE_KEY_LAST_HEARTBEAT, Date.now().toString())
			console.debug('[Analytics] Heartbeat sent successfully')
		}
	} catch (error) {
		// Тихо игнорируем ошибки аналитики
		console.debug('[Analytics] Heartbeat failed:', error.message)
	}
}

/**
 * Инициализирует анонимную аналитику
 * Вызывается при запуске приложения
 */
export function initAnonymousAnalytics() {
	console.info('[Analytics] Initializing anonymous analytics')

	// Отправляем heartbeat при запуске (если нужно)
	sendAnonymousHeartbeat()

	// Опционально: отправлять heartbeat периодически пока приложение открыто
	// setInterval(sendAnonymousHeartbeat, HEARTBEAT_INTERVAL)
}

/**
 * Получает статистику использования (для отладки)
 */
export function getAnalyticsInfo() {
	return {
		anonymousId: getOrCreateAnonymousId(),
		lastHeartbeat: localStorage.getItem(STORAGE_KEY_LAST_HEARTBEAT),
		nextHeartbeat: shouldSendHeartbeat() ? 'Now' : 'Later',
	}
}
