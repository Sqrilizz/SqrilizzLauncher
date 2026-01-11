<script setup lang="ts" generic="T">
import type { RouteLocationRaw } from 'vue-router'

import AutoLink from '../base/AutoLink.vue'
import Avatar from '../base/Avatar.vue'
import Checkbox from '../base/Checkbox.vue'

export interface ContentCreator {
	name: string
	type: 'user' | 'organization'
	id: string
	link?: string | RouteLocationRaw
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	linkProps?: any
}

export interface ContentProject {
	id: string
	link?: string | RouteLocationRaw
	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	linkProps?: any
}

export interface ContentItem<T> {
	path: string
	disabled: boolean
	filename: string
	data: T

	icon?: string
	title?: string
	project?: ContentProject
	creator?: ContentCreator

	version?: string
	versionId?: string
}

withDefaults(
	defineProps<{
		item: ContentItem<T>
		last?: boolean
	}>(),
	{
		last: false,
	},
)

const model = defineModel<boolean>()
</script>

<template>
	<div
		class="content-list-item grid grid-cols-[min-content,1fr,auto] gap-2 items-center px-2 py-0 min-h-[22px] border-solid border-0 border-b-button-bg relative"
		:class="{
			'border-b-[1px]': !last,
			'opacity-60': item.disabled
		}"
	>
		<!-- Checkbox -->
		<Checkbox v-model="model" :description="``" class="select-checkbox flex-shrink-0" />

		<!-- Icon + Title + Version -->
		<div class="flex items-center gap-2 min-w-0">
			<AutoLink
				:to="item.project?.link ?? ''"
				tabindex="-1"
				v-bind="item.project?.linkProps ?? {}"
				class="flex-shrink-0"
			>
				<Avatar
					:src="item.icon ?? ''"
					:class="{ grayscale: item.disabled }"
					size="28px"
					class="rounded-md shadow-sm"
				/>
			</AutoLink>

			<div class="flex flex-col gap-0 min-w-0 flex-1">
				<div class="flex items-center gap-1.5 min-w-0">
					<AutoLink :to="item.project?.link ?? ''" v-bind="item.project?.linkProps ?? {}" class="min-w-0">
						<span
							class="text-contrast font-medium text-xs leading-tight hover:text-brand"
							:class="{ 'line-through': item.disabled }"
						>
							{{ item.title ?? item.filename }}
						</span>
					</AutoLink>
					<span v-if="item.version" class="text-secondary text-[9px] flex-shrink-0">|</span>
					<span v-if="item.version" class="text-secondary text-[9px] font-mono whitespace-nowrap">
						{{ item.version }}
					</span>
				</div>

				<AutoLink
					v-if="item.creator && item.creator.name"
					:to="item.creator?.link ?? ''"
					v-bind="item.creator?.linkProps ?? {}"
				>
					<span class="text-secondary text-[10px] leading-tight hover:text-contrast">
						by {{ item.creator.name }}
					</span>
				</AutoLink>
			</div>
		</div>

		<!-- Actions -->
		<div class="flex justify-end gap-0 items-center flex-shrink-0">
			<slot name="actions" :item="item" />
		</div>
	</div>
</template>

<style scoped>
.content-list-item {
	background: transparent;
}

.content-list-item:hover {
	background: var(--color-surface-3);
}

/* Compact checkbox */
.select-checkbox :deep(button.checkbox) {
	width: 1.25rem;
	height: 1.25rem;
	border-radius: 0.25rem;
}
</style>
