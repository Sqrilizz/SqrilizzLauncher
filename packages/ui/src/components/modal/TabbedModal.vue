<script setup lang="ts">
import { type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { type Component, nextTick, ref, watch } from 'vue'

const { formatMessage } = useVIntl()

export type Tab<Props> = {
	name: MessageDescriptor
	icon: Component
	content: Component<Props>
	props?: Props
}

defineProps<{
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	tabs: Tab<any>[]
}>()

const selectedTab = ref(0)
const content = ref<HTMLElement | null>(null)

function setTab(index: number) {
	selectedTab.value = index
}

watch(selectedTab, async () => {
	await nextTick()
	content.value?.scrollTo({ top: 0, behavior: 'instant' })
})

defineExpose({ selectedTab, setTab })
</script>
<template>
	<div class="tabbed-modal">
		<aside class="tabbed-modal__sidebar">
			<div class="tabbed-modal__eyebrow">SETTINGS</div>
			<div class="tabbed-modal__title">Configure your launcher</div>
			<nav class="tabbed-modal__nav" aria-label="Settings sections">
				<button
					v-for="(tab, index) in tabs"
					:key="index"
					:class="['tabbed-modal__tab', { 'is-selected': selectedTab === index }]"
					:aria-current="selectedTab === index ? 'page' : undefined"
					@click="() => (selectedTab = index)"
				>
					<span class="tabbed-modal__tab-icon"><component :is="tab.icon" /></span>
					<span class="tabbed-modal__tab-label">{{ formatMessage(tab.name) }}</span>
					<span class="tabbed-modal__tab-arrow" aria-hidden="true">›</span>
				</button>
			</nav>
			<div class="tabbed-modal__sidebar-footer">
				<span class="tabbed-modal__status-dot" aria-hidden="true" />
				Changes save automatically
			</div>
			<div v-if="$slots.footer" class="tabbed-modal__footer">
				<slot name="footer" />
			</div>
		</aside>
		<main ref="content" class="tabbed-modal__content">
			<component :is="tabs[selectedTab].content" v-bind="tabs[selectedTab].props ?? {}" />
		</main>
	</div>
</template>

<style scoped lang="scss">
.tabbed-modal {
	display: grid;
	grid-template-columns: minmax(13rem, 15rem) minmax(0, 1fr);
	width: min(56rem, calc(100vw - 3rem));
	min-height: min(36rem, calc(100vh - 8rem));
	background: var(--color-bg-raised);
}

.tabbed-modal__sidebar {
	display: flex;
	flex-direction: column;
	gap: 0.45rem;
	padding: 1.25rem 0.9rem 1.1rem;
	border-right: 1px solid var(--color-divider);
	background: color-mix(in srgb, var(--color-bg) 62%, var(--color-bg-raised));
}

.tabbed-modal__eyebrow {
	padding: 0 0.7rem;
	color: var(--color-brand);
	font-size: 0.65rem;
	font-weight: 800;
	letter-spacing: 0.14em;
}

.tabbed-modal__title {
	padding: 0 0.7rem 0.8rem;
	color: var(--color-contrast);
	font-size: 0.9rem;
	font-weight: 750;
}

.tabbed-modal__nav {
	display: flex;
	flex-direction: column;
	gap: 0.25rem;
	min-height: 0;
	overflow-y: auto;
	overscroll-behavior: contain;
}

.tabbed-modal__tab {
	display: flex;
	align-items: center;
	gap: 0.7rem;
	min-height: 2.75rem;
	padding: 0.6rem 0.7rem;
	border: 1px solid transparent;
	border-radius: 0.8rem;
	background: transparent;
	color: var(--color-button-text);
	font: inherit;
	font-size: 0.82rem;
	font-weight: 650;
	text-align: left;
	cursor: pointer;
	transition:
		background 160ms ease,
		border-color 160ms ease,
		color 160ms ease,
		transform 160ms ease;
}

.tabbed-modal__tab:hover {
	background: var(--color-button-bg);
	color: var(--color-contrast);
}

.tabbed-modal__tab:active {
	transform: scale(0.98);
}

.tabbed-modal__tab.is-selected {
	border-color: color-mix(in srgb, var(--color-brand) 42%, transparent);
	background: color-mix(in srgb, var(--color-brand) 18%, var(--color-button-bg));
	color: var(--color-brand);
}

.tabbed-modal__tab:focus-visible {
	outline: 2px solid var(--color-brand);
	outline-offset: 2px;
}

.tabbed-modal__tab-icon {
	display: inline-flex;
	align-items: center;
	justify-content: center;
	width: 1.45rem;
	height: 1.45rem;
	flex: 0 0 1.45rem;
	border-radius: 0.45rem;
	background: var(--color-button-bg);
	color: var(--color-secondary);
}

.tabbed-modal__tab-icon :deep(svg) {
	width: 0.9rem;
	height: 0.9rem;
}

.tabbed-modal__tab.is-selected .tabbed-modal__tab-icon {
	background: var(--color-brand);
	color: var(--color-bg);
}

.tabbed-modal__tab-label {
	min-width: 0;
	flex: 1;
}

.tabbed-modal__tab-arrow {
	color: var(--color-secondary);
	font-size: 1.2rem;
	line-height: 1;
	opacity: 0;
	transform: translateX(-0.2rem);
	transition:
		opacity 160ms ease,
		transform 160ms ease;
}

.tabbed-modal__tab:hover .tabbed-modal__tab-arrow,
.tabbed-modal__tab.is-selected .tabbed-modal__tab-arrow {
	opacity: 1;
	transform: translateX(0);
}

.tabbed-modal__sidebar-footer {
	display: flex;
	align-items: center;
	gap: 0.45rem;
	margin-top: auto;
	padding: 0.8rem 0.7rem 0;
	border-top: 1px solid var(--color-divider);
	color: var(--color-secondary);
	font-size: 0.68rem;
	line-height: 1.35;
}

.tabbed-modal__footer {
	margin-top: auto;
	padding: 0.2rem 0;
}

.tabbed-modal__status-dot {
	width: 0.45rem;
	height: 0.45rem;
	flex: 0 0 0.45rem;
	border-radius: 999px;
	background: var(--color-green);
	box-shadow: 0 0 0.55rem color-mix(in srgb, var(--color-green) 65%, transparent);
}

.tabbed-modal__content {
	min-width: 0;
	max-height: min(36rem, calc(100vh - 8rem));
	padding: 1.5rem 1.7rem;
	overflow-y: auto;
	overscroll-behavior: contain;
	scrollbar-gutter: stable;
}

.tabbed-modal__content :deep(h2) {
	margin-top: 1.5rem;
	margin-bottom: 0.4rem;
	color: var(--color-contrast);
	font-size: 1.05rem;
	font-weight: 800;
	letter-spacing: -0.02em;
}

.tabbed-modal__content :deep(h2:first-child) {
	margin-top: 0;
}

.tabbed-modal__content :deep(p) {
	color: var(--color-secondary);
	font-size: 0.82rem;
	line-height: 1.45;
}

.tabbed-modal__content :deep(input:not([type='checkbox']):not([type='radio']):not([type='range'])),
.tabbed-modal__content :deep(textarea),
.tabbed-modal__content :deep(select) {
	min-height: 2.5rem;
	padding: 0.55rem 0.8rem;
	border-color: var(--color-divider);
	border-radius: 0.7rem;
	background: var(--color-button-bg);
}

.tabbed-modal__content
	:deep(input:not([type='checkbox']):not([type='radio']):not([type='range']):focus),
.tabbed-modal__content :deep(textarea:focus),
.tabbed-modal__content :deep(select:focus) {
	border-color: var(--color-brand);
	box-shadow: 0 0 0 3px color-mix(in srgb, var(--color-brand) 16%, transparent);
	outline: none;
}

.tabbed-modal__content :deep(input[type='range']) {
	min-height: 0;
	padding: 0;
	border: 0;
	box-shadow: none;
}

.tabbed-modal__content :deep(.flex.items-center.justify-between) {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1.25rem;
	margin-top: 0.75rem;
	padding: 0.9rem 1rem;
	border: 1px solid var(--color-divider);
	border-radius: 0.9rem;
	background: color-mix(in srgb, var(--color-bg) 48%, var(--color-bg-raised));
}

.tabbed-modal__content :deep(.flex.items-center.justify-between > div:first-child) {
	min-width: 0;
	flex: 1;
}

.tabbed-modal__content :deep(.flex.items-center.justify-between > :last-child) {
	flex: 0 0 auto;
}

.tabbed-modal__content :deep(.flex.items-center.justify-between > .relative.inline-block.w-full) {
	width: 10rem;
	flex: 0 0 10rem;
}

.tabbed-modal__content :deep(.flex.items-center.justify-between h2),
.tabbed-modal__content :deep(.flex.items-center.justify-between h3) {
	margin-top: 0;
	margin-bottom: 0.25rem;
	font-size: 0.9rem;
	line-height: 1.25;
}

.tabbed-modal__content :deep(.flex.items-center.justify-between p) {
	max-width: 38rem;
	margin-top: 0.25rem;
	font-size: 0.76rem;
}

.tabbed-modal__content :deep(hr) {
	height: 1px;
	margin: 1.25rem 0;
	border: 0;
	background: var(--color-divider);
}

.tabbed-modal__content :deep(.modal-body) {
	padding: 0.9rem;
	border: 1px solid color-mix(in srgb, var(--color-brand) 28%, var(--color-divider));
	border-radius: 0.9rem;
	background: color-mix(in srgb, var(--color-brand) 7%, var(--color-bg));
}

.tabbed-modal__content :deep(button:focus-visible),
.tabbed-modal__content :deep([role='button']:focus-visible) {
	outline: 2px solid var(--color-brand);
	outline-offset: 2px;
}

@media (max-width: 760px) {
	.tabbed-modal {
		grid-template-columns: 1fr;
		width: min(40rem, calc(100vw - 2rem));
		min-height: 0;
	}

	.tabbed-modal__sidebar {
		border-right: 0;
		border-bottom: 1px solid var(--color-divider);
	}

	.tabbed-modal__title,
	.tabbed-modal__eyebrow,
	.tabbed-modal__sidebar-footer {
		display: none;
	}

	.tabbed-modal__nav {
		flex-direction: row;
		overflow-x: auto;
		padding-bottom: 0.15rem;
	}

	.tabbed-modal__tab {
		flex: 0 0 auto;
	}

	.tabbed-modal__tab-arrow {
		display: none;
	}

	.tabbed-modal__content {
		max-height: 60vh;
		padding: 1.25rem;
	}
}
</style>
