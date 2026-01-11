<script setup>
import {
    CheckIcon,
    CopyIcon,
    DropdownIcon,
    HammerIcon,
    LogInIcon,
    UpdatedIcon,
    XIcon,
} from '@modrinth/assets'
import { ButtonStyled, Collapsible, injectNotificationManager } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { ChatIcon } from '@/assets/icons'
import ModalWrapper from '@/components/ui/modal/ModalWrapper.vue'
import { trackEvent } from '@/helpers/analytics'
import { login as login_flow, set_default_user } from '@/helpers/auth.js'
import { install } from '@/helpers/profile.js'
import { cancel_directory_change } from '@/helpers/settings.ts'
import { handleSevereError } from '@/store/error.js'

// [AR] Imports
import { applyMigrationFix, restartApp } from '@/helpers/utils.js'

const { handleError } = injectNotificationManager()

const errorModal = ref()
const error = ref()
const closable = ref(true)
const errorCollapsed = ref(false)
const migrationFixSuccess = ref(null) // null | true | false
const migrationFixCallbackModel = ref()

const title = ref('An error occurred')
const errorType = ref('unknown')
const supportLink = ref('https://support.modrinth.com')
const metadata = ref({})

defineExpose({
	async show(errorVal, context, canClose = true, source = null) {
		closable.value = canClose

		if (errorVal.message && errorVal.message.includes('Minecraft authentication error:')) {
			title.value = 'Unable to sign in to Minecraft'
			errorType.value = 'minecraft_auth'
			supportLink.value =
				'https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues'

			if (
				errorVal.message.includes('existing connection was forcibly closed') ||
				errorVal.message.includes('error sending request for url')
			) {
				metadata.value.network = true
			}
			if (errorVal.message.includes('because the target machine actively refused it')) {
				metadata.value.hostsFile = true
			}
		} else if (errorVal.message && errorVal.message.includes('User is not logged in')) {
			title.value = 'Sign in to Minecraft'
			errorType.value = 'minecraft_sign_in'
			supportLink.value = 'https://support.modrinth.com'
		} else if (errorVal.message && errorVal.message.includes('Move directory error:')) {
			title.value = 'Could not change app directory'
			errorType.value = 'directory_move'
			supportLink.value = 'https://support.modrinth.com'

			if (errorVal.message.includes('directory is not writeable')) {
				metadata.value.readOnly = true
			}

			if (errorVal.message.includes('Not enough space')) {
				metadata.value.notEnoughSpace = true
			}
		} else if (errorVal.message && errorVal.message.includes('No loader version selected for')) {
			title.value = 'No loader selected'
			errorType.value = 'no_loader_version'
			supportLink.value = 'https://support.modrinth.com'
			metadata.value.profilePath = context.profilePath
		} else if (source === 'state_init') {
			title.value = 'Error initializing Sqrilizz Launcher'
			errorType.value = 'state_init'
			supportLink.value = 'https://support.modrinth.com'
		} else {
			title.value = 'An error occurred'
			errorType.value = 'unknown'
			supportLink.value = 'https://support.modrinth.com'
			metadata.value = {}
		}

		error.value = errorVal
		errorModal.value.show()
	},
})

const loadingMinecraft = ref(false)
async function loginMinecraft() {
	try {
		loadingMinecraft.value = true
		const loggedIn = await login_flow()

		if (loggedIn) {
			await set_default_user(loggedIn.profile.id).catch(handleError)
		}

		await trackEvent('AccountLogIn', { source: 'ErrorModal' })
		loadingMinecraft.value = false
		errorModal.value.hide()
	} catch (err) {
		loadingMinecraft.value = false
		handleSevereError(err)
	}
}

async function cancelDirectoryChange() {
	try {
		await cancel_directory_change()
		window.location.reload()
	} catch (err) {
		handleError(err)
	}
}

function retryDirectoryChange() {
	window.location.reload()
}

const loadingRepair = ref(false)
async function repairInstance() {
	loadingRepair.value = true
	try {
		await install(metadata.value.profilePath, false)
		errorModal.value.hide()
	} catch (err) {
		handleSevereError(err)
	}
	loadingRepair.value = false
}

const hasDebugInfo = computed(
	() =>
		errorType.value === 'directory_move' ||
		errorType.value === 'minecraft_auth' ||
		errorType.value === 'state_init' ||
		errorType.value === 'no_loader_version',
)

const debugInfo = computed(() => error.value.message ?? error.value ?? 'No error message.')

const copied = ref(false)

async function copyToClipboard(text) {
	await navigator.clipboard.writeText(text)
	copied.value = true
	setTimeout(() => {
		copied.value = false
	}, 3000)
}

async function onApplyMigrationFix(eol) {
  try {
    const result = await applyMigrationFix(eol)
    migrationFixSuccess.value = result === true
  } catch (err) {
    console.error(`Failed to apply migration fix:`, err)
    migrationFixSuccess.value = false
  } finally {
    migrationFixCallbackModel.value?.show?.()
    if (migrationFixSuccess.value === true) {
      setTimeout(async () => {
        await restartApp()
      }, 3000)
    }
  }
}

</script>

<template>
	<ModalWrapper ref="errorModal" :header="title" :closable="closable">
		<div class="modal-body">
			<div class="markdown-body">
				<template v-if="errorType === 'minecraft_auth'">
					<template v-if="metadata.network">
						<h3>Network issues</h3>
						<p>
							It looks like there were issues with the Sqrilizz Launcher connecting to Microsoft's
							servers. This is often the result of a poor connection, so we recommend trying again
							to see if it works. If issues continue to persist, follow the steps in
							<a
								href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_e71a5f805f"
							>
								our support article
							</a>
							to troubleshoot.
						</p>
					</template>
					<template v-else-if="metadata.hostsFile">
						<h3>Network issues</h3>
						<p>
							The Sqrilizz Launcher tried to connect to Microsoft / Xbox / Minecraft services, but the
							remote server rejected the connection. This may indicate that these services are
							blocked by the hosts file. Please visit
							<a
								href="https://support.modrinth.com/en/articles/9038231-minecraft-sign-in-issues#h_d694a29256"
							>
								our support article
							</a>
							for steps on how to fix the issue.
						</p>
					</template>
					<template v-else>
						<h3>Try another Microsoft account</h3>
						<p>
							Double check you've signed in with the right account. You may own Minecraft on a
							different Microsoft account.
						</p>
						<div class="cta-button">
							<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
								<LogInIcon /> Try another account
							</button>
						</div>
						<h3>Using PC Game Pass, coming from Bedrock, or just bought the game?</h3>
						<p>
							Try signing in with the
							<a href="https://www.minecraft.net/en-us/download">official Minecraft Launcher</a>
							first. Once you're done, come back here and sign in!
						</p>
					</template>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
							<LogInIcon /> Try signing in again
						</button>
					</div>
				</template>
				<template v-if="errorType === 'directory_move'">
					<template v-if="metadata.readOnly">
						<h3>Change directory permissions</h3>
						<p>
							It looks like the Sqrilizz Launcher is unable to write to the directory you selected.
							Please adjust the permissions of the directory and try again or cancel the directory
							change.
						</p>
					</template>
					<template v-else-if="metadata.notEnoughSpace">
						<h3>Not enough space</h3>
						<p>
							It looks like there is not enough space on the disk containing the directory you
							selected. Please free up some space and try again or cancel the directory change.
						</p>
					</template>
					<template v-else>
						<p>
							The Sqrilizz Launcher is unable to migrate to the new directory you selected. Please
							contact support for help or cancel the directory change.
						</p>
					</template>

					<div class="cta-button">
						<button class="btn" @click="retryDirectoryChange">
							<UpdatedIcon /> Retry directory change
						</button>
						<button class="btn btn-danger" @click="cancelDirectoryChange">
							<XIcon /> Cancel directory change
						</button>
					</div>
				</template>
				<div v-else-if="errorType === 'minecraft_sign_in'">
					<p>
						To play this instance, you must sign in through Microsoft below. If you don't have a
						Minecraft account, you can purchase the game on the
						<a href="https://www.minecraft.net/en-us/store/minecraft-java-bedrock-edition-pc"
							>Minecraft website</a
						>.
					</p>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingMinecraft" @click="loginMinecraft">
							<LogInIcon /> Sign in to Minecraft
						</button>
					</div>
				</div>
				<template v-else-if="errorType === 'state_init'">
					<p>
						Sqrilizz Launcher failed to load correctly. This may be because of a corrupted file, or
						because the app is missing crucial files.
					</p>
					<p>You may be able to fix it through one of the following ways:</p>
					<ul>
						<li>Ensuring you are connected to the internet, then try restarting the app.</li>
						<li>Redownloading the app.</li>
					</ul>
				</template>
				<template v-else-if="errorType === 'no_loader_version'">
					<p>The Sqrilizz Launcher failed to find the loader version for this instance.</p>
					<p>To resolve this, you need to repair the instance. Click the button below to do so.</p>
					<div class="cta-button">
						<button class="btn btn-primary" :disabled="loadingRepair" @click="repairInstance">
							<HammerIcon /> Repair instance
						</button>
					</div>
				</template>
				<template v-else>
					{{ debugInfo }}
				</template>
				<template v-if="hasDebugInfo">
					<hr />
					<p>
						If nothing is working and you need help, visit
						<a :href="supportLink">our support page</a>
						and start a chat using the widget in the bottom right and we will be more than happy to
						assist! Make sure to provide the following debug information to the agent:
					</p>
				</template>
			</div>
			<div class="flex items-center gap-2">
				<ButtonStyled>
					<a :href="supportLink" @click="errorModal.value.hide()"><ChatIcon /> Get support</a>
				</ButtonStyled>
				<ButtonStyled v-if="closable">
					<button @click="errorModal.value.hide()"><XIcon /> Close</button>
				</ButtonStyled>
				<ButtonStyled v-if="hasDebugInfo">
					<button :disabled="copied" @click="copyToClipboard(debugInfo)">
						<template v-if="copied"> <CheckIcon class="text-green" /> Copied! </template>
						<template v-else> <CopyIcon /> Copy debug info </template>
					</button>
				</ButtonStyled>
			</div>
			<template v-if="hasDebugInfo">
				<div class="bg-button-bg rounded-xl mt-2 overflow-clip">
					<button
						class="flex items-center justify-between w-full bg-transparent border-0 px-4 py-3 cursor-pointer"
						@click="errorCollapsed = !errorCollapsed"
					>
						<span class="text-contrast font-extrabold m-0">Debug information:</span>
						<DropdownIcon
							class="h-5 w-5 text-secondary transition-transform"
							:class="{ 'rotate-180': !errorCollapsed }"
						/>
					</button>
					<Collapsible :collapsed="errorCollapsed">
						<pre class="m-0 px-4 py-3 bg-bg rounded-none whitespace-pre-wrap break-words overflow-x-auto max-w-full"
							>{{ debugInfo }}</pre>
					</Collapsible>
				</div>
				<template v-if="errorType === 'state_init'">
        			<h2>⚠️ Migration Issue • Important Notice</h2>
        			<p>We've detected a problem with our database migration system caused by inconsistent line endings between operating systems (Windows vs. macOS/Linux). This may affect app stability.</p>
        			<p><strong>What’s happening?</strong> Our migration validator misreads modified migrations when line endings differ (CRLF ↔ LF), which can make the app unusable.</p>
        			<p><strong>Why?</strong> Git’s automatic line-ending conversions and OS differences can cause these inconsistencies during builds.</p>
        			<p><strong>What’s next?</strong> We’re working on a permanent fix. In the meantime, you can apply one of the quick fixes below depending on your system.</p>
        			<h3>Do I need to apply a fix now?</h3>
					<div>
						<p class="notice__text">
							If you're encountering an error while applying migrations, such as "Error while applying migrations: migration XXXXXXXXXX was previously applied but has been modified", or a similar issue with migration, the following actions might help:
						</p>
						<p>If none of the above steps help, you can try saving a copy of the file <code>app.db</code> to a safe location, such as <code>%appdata%\Roaming\SqrilizzLauncher</code>
							on Windows or <code>~/Library/Application Support/SqrilizzLauncher</code> on macOS, then deleting the original file and letting the app re-create the database file.
							Note that this may cause data loss inside the app, so make sure to back up your launcher data before applying this fixes.
						</p>
					</div>
        		    <div class="flex justify-between">
        		      <ol class="flex flex-col gap-3">
        		        <li>
        		          <ButtonStyled class="neon-button neon">
        		            <button
        		              title="Convert all line endings in migration files to LF (Unix-style: \\n)"
        		              @click="onApplyMigrationFix('lf')"
        		            >
        		            	Apply fix for Unix like systems (Debian, Ubuntu, macOS and others)
        		            </button>
        		          </ButtonStyled>
        		        </li>
        		        <li>
        		          <ButtonStyled class="neon-button neon">
        		            <button
        		              title="Convert all line endings in migration files to CRLF (Windows-style: \\r\\n)"
        		              @click="onApplyMigrationFix('crlf')"
        		            >
								Apply fix for Windows
        		            </button>
        		          </ButtonStyled>
        		        </li>
        		      </ol>
        		    </div>
        		</template>
			</template>
		</div>
	</ModalWrapper>
	<ModalWrapper
  	  ref="migrationFixCallbackModel"
  	  header="💡 Migration fix report"
  	  :closable="closable">
  	  <div class="modal-body">
  	    <h2 class="text-lg font-bold text-contrast space-y-2">
  	      <template v-if="migrationFixSuccess === true">
  	        <p class="flex items-center gap-2 neon-text">
  	        	✅ The migration fix has been applied successfully. Please restart the launcher and try to log in to the game :)
  	        </p>
  	        <p class="mt-2 text-sm neon-text">
				If the problem persists, please try the other fix.
  	        </p>
  	      </template>

  	      <template v-else-if="migrationFixSuccess === false">
  	        <p class="flex items-center gap-2 neon-text">
  	        	❌ The migration fix failed or had no effect.
  	        </p>
  	        <p class="mt-2 text-sm neon-text">
				If the problem persists, please try the other fix.
  	        </p>
  	      </template>
  	    </h2>
  	  </div>
  	</ModalWrapper>
</template>

<style>
.light-mode {
	--color-orange-bg: rgba(255, 163, 71, 0.2);
}

.dark-mode,
.oled-mode {
	--color-orange-bg: rgba(224, 131, 37, 0.2);
}
</style>

<style scoped lang="scss">
@import '../../../../../packages/assets/styles/neon-button.scss';
@import '../../../../../packages/assets/styles/neon-text.scss';

code {
  background: linear-gradient(90deg, #005eff, #00cfff);
  background-clip: text;
  -webkit-background-clip: text;
  color: transparent;
}

.cta-button {
	display: flex;
	align-items: center;
	justify-content: center;
	padding: 0.5rem;
	gap: 0.5rem;
}

.warning-banner {
	display: flex;
	flex-direction: column;
	gap: 0.5rem;
	padding: var(--gap-lg);
	background-color: var(--color-orange-bg);
	border: 2px solid var(--color-orange);
	border-radius: var(--radius-md);
	margin-bottom: 1rem;
}

.warning-banner__title {
	display: flex;
	align-items: center;
	gap: 0.5rem;
	font-weight: 700;

	svg {
		color: var(--color-orange);
		height: 1.5rem;
		width: 1.5rem;
	}
}

.modal-body {
	display: flex;
	flex-direction: column;
	gap: var(--gap-md);
}

.markdown-body {
	overflow: auto;
}
</style>
