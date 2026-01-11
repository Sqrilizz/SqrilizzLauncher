<template>
<Card class="log-card">
<div class="button-row">
<DropdownSelect
v-model="selectedLogIndex"
:default-value="0"
name="Log date"
:options="logs.map((_, index) => index)"
:display-name="(option) => logs[option]?.name"
:disabled="logs.length === 0"
/>
<div class="button-group">
<Button :disabled="!logs[selectedLogIndex]" @click="copyLog()">
<ClipboardCopyIcon v-if="!copied" />
<CheckIcon v-else />
{{ copied ? 'Copied' : 'Copy' }}
</Button>
<Button
v-if="logs[selectedLogIndex] && logs[selectedLogIndex].live === true"
@click="clearLiveLog()"
>
<TrashIcon aria-hidden="true" />
Clear
</Button>
<Button
v-else
:disabled="!logs[selectedLogIndex] || logs[selectedLogIndex].live === true"
color="danger"
@click="deleteLog()"
>
<TrashIcon aria-hidden="true" />
Delete
</Button>
</div>
</div>
<div class="button-row">
<input
id="text-filter"
v-model="searchFilter"
autocomplete="off"
type="text"
class="text-filter"
placeholder="Type to filter logs..."
/>
</div>
<div ref="logContainer" class="log-text">
<div v-if="loading" class="log-loading">Loading logs...</div>
<div v-else-if="error" class="log-error">{{ error }}</div>
<div v-else-if="displayLines.length === 0" class="log-empty">No logs available</div>
<div v-else class="log-lines">
<div v-for="line in displayLines" :key="line.id" class="log-line">
<span v-if="line.prefix" :style="{ color: line.prefixColor, fontWeight: line.weight }">
{{ line.prefix }}
</span>
<span :style="{ color: line.textColor }">{{ line.text }}</span>
</div>
</div>
</div>
</Card>
</template>

<script setup>
import { CheckIcon, ClipboardCopyIcon, TrashIcon } from '@modrinth/assets'
import { Button, Card, DropdownSelect } from '@modrinth/ui'
import { computed, onUnmounted, ref, watch } from 'vue'
import { useRoute } from 'vue-router'

import { process_listener } from '@/helpers/events.js'
import {
delete_logs_by_filename,
get_latest_log_cursor,
get_logs,
get_output_by_filename,
} from '@/helpers/logs.js'
import { get_by_profile_path } from '@/helpers/process.js'

const route = useRoute()

const props = defineProps({
instance: {
type: Object,
default() {
return {}
},
},
playing: {
type: Boolean,
default() {
return false
},
},
})

const logs = ref([])
const loading = ref(true)
const error = ref(null)
const selectedLogIndex = ref(0)
const copied = ref(false)
const logContainer = ref(null)
const searchFilter = ref('')
const currentLiveLog = ref('')
const currentLiveLogCursor = ref(0)

const displayLines = computed(() => {
const currentLog = logs.value[selectedLogIndex.value]
if (!currentLog || !currentLog.stdout) return []

const lines = currentLog.stdout.split('\n')
const processed = []
let id = 0

for (const line of lines) {
if (searchFilter.value && !line.toLowerCase().includes(searchFilter.value.toLowerCase())) {
continue
}

const prefix = line.includes(']:') ? line.split(']:')[0] + ']:' : null
const text = prefix ? line.substring(prefix.length) : line

let prefixColor = 'var(--color-blue)'
let textColor = 'var(--color-white)'
let weight = 'normal'

if (line.includes('/ERROR') || line.includes('Exception:') || line.includes('Error')) {
prefixColor = 'var(--color-red)'
textColor = 'var(--color-red)'
weight = 'bold'
} else if (line.includes('/WARN')) {
prefixColor = 'var(--color-orange)'
textColor = 'var(--color-orange)'
weight = 'bold'
} else if (line.trim().startsWith('#')) {
prefixColor = 'var(--color-green)'
textColor = 'var(--color-green)'
}

processed.push({ id: id++, prefix, text, prefixColor, textColor, weight })
}

return processed
})

async function getLiveLog() {
try {
if (!route.params.id) return null

const processes = await get_by_profile_path(route.params.id)
if (!processes || processes.length === 0) {
return { name: 'Live Log', stdout: 'No live game detected.\nStart your game to proceed.', live: true }
}

const logCursor = await get_latest_log_cursor(props.instance.path, currentLiveLogCursor.value)
if (logCursor.new_file) {
currentLiveLog.value = ''
}
currentLiveLog.value += logCursor.output
currentLiveLogCursor.value = logCursor.cursor

return { name: 'Live Log', stdout: currentLiveLog.value, live: true }
} catch (err) {
console.error('[Logs] Error getting live log:', err)
return { name: 'Live Log', stdout: 'Error loading live log', live: true }
}
}

async function loadLogs() {
try {
loading.value = true
error.value = null

const [liveLog, allLogs] = await Promise.all([
getLiveLog(),
get_logs(props.instance.path, true)
])

const filteredLogs = (allLogs || [])
.filter(log => 
log.filename !== 'latest_stdout.log' &&
log.filename !== 'latest_stdout' &&
log.stdout !== '' &&
(log.filename.includes('.log') || log.filename.endsWith('.txt'))
)
.map(log => ({
...log,
name: log.filename || 'Unknown',
}))

logs.value = [liveLog, ...filteredLogs].filter(Boolean)

if (logs.value.length > 1 && !props.playing) {
selectedLogIndex.value = 1
} else {
selectedLogIndex.value = 0
}
} catch (err) {
console.error('[Logs] Error loading logs:', err)
error.value = 'Failed to load logs'
} finally {
loading.value = false
}
}

const copyLog = () => {
const currentLog = logs.value[selectedLogIndex.value]
if (currentLog && currentLog.stdout) {
navigator.clipboard.writeText(currentLog.stdout)
copied.value = true
setTimeout(() => { copied.value = false }, 2000)
}
}

const deleteLog = async () => {
const currentLog = logs.value[selectedLogIndex.value]
if (!currentLog || selectedLogIndex.value === 0) return

try {
await delete_logs_by_filename(
props.instance.path,
currentLog.log_type,
currentLog.filename
)
selectedLogIndex.value = Math.max(0, selectedLogIndex.value - 1)
await loadLogs()
} catch (err) {
console.error('[Logs] Error deleting log:', err)
}
}

const clearLiveLog = () => {
currentLiveLog.value = ''
}

watch(selectedLogIndex, async (newIndex) => {
copied.value = false
const log = logs.value[newIndex]

if (log && !log.live && newIndex !== 0 && log.stdout === undefined) {
try {
log.stdout = 'Loading...'
log.stdout = await get_output_by_filename(
props.instance.path,
log.log_type,
log.filename
)
} catch (err) {
console.error('[Logs] Error loading log content:', err)
log.stdout = 'Error loading log'
}
}
})

const interval = setInterval(async () => {
if (selectedLogIndex.value === 0 && logs.value.length > 0) {
const liveLog = await getLiveLog()
if (liveLog) {
logs.value[0] = liveLog

if (logContainer.value) {
const { scrollTop, scrollHeight, clientHeight } = logContainer.value
const isNearBottom = scrollHeight - scrollTop - clientHeight < 100
if (isNearBottom) {
setTimeout(() => {
if (logContainer.value) {
logContainer.value.scrollTop = logContainer.value.scrollHeight
}
}, 50)
}
}
}
}
}, 250)

let unlistenProcesses = null
process_listener(async (e) => {
if (e.event === 'launched') {
currentLiveLog.value = ''
currentLiveLogCursor.value = 0
selectedLogIndex.value = 0
}
if (e.event === 'finished') {
currentLiveLog.value = ''
currentLiveLogCursor.value = 0
await loadLogs()
if (logs.value.length > 1) {
selectedLogIndex.value = 1
}
}
}).then(unlisten => { unlistenProcesses = unlisten })

loadLogs()

onUnmounted(() => {
clearInterval(interval)
if (unlistenProcesses) unlistenProcesses()
})
</script>

<style scoped lang="scss">
.log-card {
display: flex;
flex-direction: column;
gap: 1rem;
height: 100%;
padding: 1rem;
}

.button-row {
display: flex;
flex-direction: row;
justify-content: space-between;
gap: 0.5rem;
flex-wrap: wrap;
}

.button-group {
display: flex;
flex-direction: row;
gap: 0.5rem;
}

.text-filter {
flex: 1;
min-width: 200px;
padding: 0.5rem;
border-radius: var(--radius-md);
background-color: var(--color-bg);
color: var(--color-contrast);
border: 1px solid var(--color-button-bg);
}

.log-text {
flex: 1;
width: 100%;
font-family: var(--mono-font);
background-color: var(--color-accent-contrast);
color: var(--color-contrast);
border-radius: var(--radius-lg);
padding: 1rem;
overflow-y: auto;
overflow-x: auto;
color-scheme: dark;
}

.log-loading,
.log-error,
.log-empty {
padding: 2rem;
text-align: center;
color: var(--color-secondary);
}

.log-error {
color: var(--color-red);
}

.log-lines {
white-space: pre-wrap;
word-break: break-word;
}

.log-line {
line-height: 1.4;
user-select: text;
font-size: 0.875rem;
}
</style>
