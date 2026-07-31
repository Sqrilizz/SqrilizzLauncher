<template>
	<RouterLink
		v-if="typeof to === 'string'"
		:to="to"
		:aria-label="label"
		v-bind="$attrs"
		:class="{
			'router-link-active': isPrimary && isPrimary(route),
			'subpage-active': isSubpage && isSubpage(route),
			disabled: disabled,
		}"
		class="w-12 h-12 text-primary rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
	>
		<slot />
	</RouterLink>
	<button
		v-else
		v-bind="$attrs"
		:aria-label="label"
		class="button-animation border-none text-primary cursor-pointer w-12 h-12 rounded-full flex items-center justify-center text-2xl transition-all bg-transparent hover:bg-button-bg hover:text-contrast"
		:disabled="disabled"
		@click="to"
	>
		<slot />
	</button>
</template>

<script setup lang="ts">
import type { RouteLocationNormalizedLoaded } from 'vue-router'
import { RouterLink, useRoute } from 'vue-router'

const route = useRoute()

type RouteFunction = (route: RouteLocationNormalizedLoaded) => boolean

withDefaults(
	defineProps<{
		to: (() => void) | string
		label?: string
		isPrimary?: RouteFunction
		isSubpage?: RouteFunction
		highlightOverride?: boolean
		disabled?: boolean
	}>(),
	{
		label: '',
		isPrimary: () => false,
		isSubpage: () => false,
		highlightOverride: false,
		disabled: false,
	},
)

defineOptions({
	inheritAttrs: false,
})
</script>

<style lang="scss" scoped>
.router-link-active,
.subpage-active {
	svg {
		filter: drop-shadow(0 0 0.5rem black);
	}
}

.router-link-active {
	@apply text-[--color-button-text-selected] bg-[--color-button-bg-selected];
}

.subpage-active {
	@apply text-contrast bg-button-bg;
}

a:focus-visible,
button:focus-visible {
	outline: 0.2rem solid var(--color-brand);
	outline-offset: 0.15rem;
}
</style>
