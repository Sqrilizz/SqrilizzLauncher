<script setup>
import { SparklesIcon } from '@modrinth/assets'
import { ButtonStyled } from '@modrinth/ui'
import { ref } from 'vue'

import AccountsCard from '@/components/ui/AccountsCard.vue'

const emit = defineEmits(['complete'])
const step = ref(0)

function complete() {
	emit('complete')
}
</script>

<template>
	<div
		class="onboarding-overlay"
		role="dialog"
		aria-modal="true"
		aria-labelledby="onboarding-title"
	>
		<div class="onboarding-card">
			<div class="onboarding-glow onboarding-glow-left"></div>
			<div class="onboarding-glow onboarding-glow-right"></div>
			<div class="onboarding-header">
				<div class="onboarding-eyebrow"><SparklesIcon /> SQRLILIZZ V2</div>
				<span class="onboarding-step">0{{ step + 1 }} / 02</span>
			</div>

			<section v-if="step === 0" class="onboarding-content">
				<p class="onboarding-kicker">WELCOME TO YOUR NEW BASE</p>
				<h1 id="onboarding-title">Your Minecraft library, rebuilt.</h1>
				<p class="onboarding-lead">
					Install modpacks, keep worlds safe, and launch every profile from one calm workspace.
				</p>
				<div class="onboarding-points">
					<div><strong>01</strong><span>Profiles that remember your setup</span></div>
					<div><strong>02</strong><span>Microsoft, Ely.by, and offline accounts</span></div>
					<div><strong>03</strong><span>Offline-first library and reliable updates</span></div>
				</div>
				<div class="onboarding-actions">
					<ButtonStyled color="brand" size="large">
						<button @click="step = 1">Set up an account</button>
					</ButtonStyled>
					<button class="onboarding-skip" @click="complete">Skip for now</button>
				</div>
			</section>

			<section v-else class="onboarding-content onboarding-account-step">
				<p class="onboarding-kicker">MAKE IT YOURS</p>
				<h1 id="onboarding-title">Choose how you play.</h1>
				<p class="onboarding-lead">You can add more accounts later from the sidebar.</p>
				<AccountsCard mode="isolated" />
				<div class="onboarding-actions">
					<button class="onboarding-back" @click="step = 0">Back</button>
					<ButtonStyled color="brand" size="large">
						<button @click="complete">Enter launcher</button>
					</ButtonStyled>
				</div>
			</section>

			<p class="onboarding-footer">You can change every choice later in Settings.</p>
		</div>
	</div>
</template>

<style scoped lang="scss">
.onboarding-overlay {
	position: fixed;
	inset: 0;
	z-index: 200;
	display: grid;
	place-items: center;
	padding: 2rem;
	background: rgba(5, 7, 12, 0.82);
	backdrop-filter: blur(18px);
}

.onboarding-card {
	position: relative;
	overflow: hidden;
	width: min(100%, 46rem);
	padding: 2.5rem;
	border: 1px solid rgba(143, 117, 255, 0.35);
	border-radius: 1.75rem;
	background: linear-gradient(145deg, rgba(25, 27, 43, 0.98), rgba(13, 16, 26, 0.98));
	box-shadow:
		0 2rem 8rem rgba(0, 0, 0, 0.5),
		0 0 4rem rgba(126, 95, 255, 0.14);
}

.onboarding-header,
.onboarding-actions {
	display: flex;
	align-items: center;
	justify-content: space-between;
	gap: 1rem;
}

.onboarding-eyebrow,
.onboarding-kicker {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	margin: 0;
	color: var(--sq-cyan);
	font-size: 0.72rem;
	font-weight: 800;
	letter-spacing: 0.16em;
}

.onboarding-eyebrow svg {
	width: 1rem;
	height: 1rem;
}

.onboarding-step {
	color: var(--color-secondary);
	font-size: 0.75rem;
	font-weight: 700;
	letter-spacing: 0.12em;
}

.onboarding-content {
	position: relative;
	z-index: 1;
	padding: 5rem 0 3rem;
}

.onboarding-kicker {
	margin-bottom: 1rem;
	color: var(--sq-brand);
}

h1 {
	max-width: 34rem;
	margin: 0;
	color: var(--color-contrast);
	font-size: clamp(2.25rem, 6vw, 4.5rem);
	font-weight: 900;
	letter-spacing: -0.06em;
	line-height: 0.94;
}

.onboarding-lead {
	max-width: 34rem;
	margin: 1.5rem 0 0;
	color: var(--color-secondary);
	font-size: 1.05rem;
	line-height: 1.6;
}

.onboarding-points {
	display: grid;
	gap: 0.75rem;
	margin: 2.5rem 0;
}

.onboarding-points div {
	display: flex;
	align-items: center;
	gap: 1rem;
	padding: 0.85rem 1rem;
	border: 1px solid rgba(255, 255, 255, 0.08);
	border-radius: 0.85rem;
	background: rgba(255, 255, 255, 0.035);
	color: var(--color-secondary);
}

.onboarding-points strong {
	color: var(--sq-cyan);
	font-size: 0.75rem;
	letter-spacing: 0.12em;
}

.onboarding-skip,
.onboarding-back {
	padding: 0.75rem 0;
	border: 0;
	background: transparent;
	color: var(--color-secondary);
	cursor: pointer;
	font: inherit;
}

.onboarding-skip:hover,
.onboarding-back:hover {
	color: var(--color-contrast);
}

.onboarding-account-step {
	padding-bottom: 2rem;
}

.onboarding-account-step :deep(.account-card) {
	max-width: 100%;
	margin-top: 2rem;
}

.onboarding-account-step .onboarding-actions {
	margin-top: 2rem;
}

.onboarding-footer {
	position: relative;
	z-index: 1;
	margin: 0;
	color: var(--color-tertiary);
	font-size: 0.8rem;
}

.onboarding-glow {
	position: absolute;
	width: 16rem;
	height: 16rem;
	border-radius: 50%;
	filter: blur(40px);
	opacity: 0.16;
	pointer-events: none;
}

.onboarding-glow-left {
	top: -7rem;
	left: -6rem;
	background: var(--sq-brand);
}

.onboarding-glow-right {
	right: -7rem;
	bottom: -8rem;
	background: var(--sq-cyan);
}

@media (max-width: 640px) {
	.onboarding-overlay {
		padding: 1rem;
	}

	.onboarding-card {
		padding: 1.5rem;
	}

	.onboarding-content {
		padding: 3rem 0 2rem;
	}
}
</style>
