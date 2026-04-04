<template>
    <div class="batch-fix-entry">
        <el-button type="danger" size="small" @click="visible = true">
            批量修正分P
        </el-button>
    </div>

    <el-dialog
        v-model="visible"
        title="批量修正分P"
        width="1200px"
        top="4vh"
        append-to-body
        destroy-on-close
    >
        <div class="batch-fix-layout">
            <div class="batch-fix-main">
                <div class="batch-fix-toolbar">
                    <el-select
                        v-model="selectedUid"
                        class="tool-user-select"
                        placeholder="选择已登录用户"
                    >
                        <el-option
                            v-for="user in loginUsers"
                            :key="user.uid"
                            :label="`${user.username}${user.expired ? ' (Cookie失效)' : ''}`"
                            :value="user.uid"
                            :disabled="user.expired"
                        />
                    </el-select>

                    <el-radio-group v-model="inputMode">
                        <el-radio-button label="query">关键词搜索</el-radio-button>
                        <el-radio-button label="json">粘贴 JSON</el-radio-button>
                    </el-radio-group>

                    <el-input-number
                        v-model="delayMs"
                        :min="0"
                        :max="10000"
                        :step="100"
                        controls-position="right"
                    />
                    <span class="delay-label">延迟(ms)</span>

                    <el-checkbox v-model="previewOnly">仅预览，不提交</el-checkbox>
                </div>

                <el-input
                    v-model="rawInput"
                    type="textarea"
                    :rows="inputMode === 'query' ? 3 : 10"
                    :placeholder="inputPlaceholder"
                />

                <div class="batch-fix-toolbar second-row">
                    <el-input
                        v-model="correctTitle"
                        clearable
                        placeholder="正确标题（可空）"
                    />
                    <el-input
                        v-model="correctArtist"
                        clearable
                        placeholder="正确歌手（可空）"
                    />
                    <el-button
                        type="primary"
                        :loading="loadingRecords"
                        :disabled="!canLoad"
                        @click="loadRecords"
                    >
                        {{ inputMode === 'query' ? '联网搜索' : '读取 JSON' }}
                    </el-button>
                    <el-button
                        type="info"
                        :loading="previewLoading"
                        :disabled="!records.length || !selectedUid"
                        @click="generatePreview"
                    >
                        生成预览
                    </el-button>
                    <el-button
                        type="success"
                        :loading="processing"
                        :disabled="!selectedUid || !selectedCount || !hasPreviewableRows"
                        @click="runBatch"
                    >
                        {{ previewOnly ? '开始预览' : '开始批处理' }}
                    </el-button>
                </div>

                <div class="batch-fix-toolbar third-row">
                    <el-button size="small" @click="setAllSelected(true)">全选</el-button>
                    <el-button size="small" @click="setAllSelected(false)">全不选</el-button>
                    <span class="summary-text">
                        已加载 {{ records.length }} 条，已选 {{ selectedCount }} 条
                    </span>
                    <span class="summary-text">
                        预览完成 {{ previewReadyCount }} / {{ records.length }}
                    </span>
                </div>

                <div v-if="progress.total > 0" class="progress-block">
                    <el-progress :percentage="progressPercent" :stroke-width="16" />
                    <div class="progress-text">
                        {{ progress.done }} / {{ progress.total }}
                        <span v-if="progress.currentKey"> | 当前：{{ progress.currentKey }}</span>
                    </div>
                </div>

                <el-table
                    :data="records"
                    height="430"
                    stripe
                    border
                    class="batch-fix-table"
                >
                    <el-table-column width="56" label="选">
                        <template #default="{ row }">
                            <el-checkbox v-model="row.selected" />
                        </template>
                    </el-table-column>
                    <el-table-column prop="bvid" label="BV" width="140" />
                    <el-table-column prop="page" label="P" width="64" />
                    <el-table-column label="搜索结果" min-width="260">
                        <template #default="{ row }">
                            <div>{{ row.title }} - {{ row.artist }}</div>
                        </template>
                    </el-table-column>
                    <el-table-column label="原分P标题" min-width="280">
                        <template #default="{ row }">
                            <div>{{ row.originalTitle || '-' }}</div>
                        </template>
                    </el-table-column>
                    <el-table-column label="预览标题" min-width="280">
                        <template #default="{ row }">
                            <div>{{ row.previewTitle || '-' }}</div>
                        </template>
                    </el-table-column>
                    <el-table-column label="状态" width="120">
                        <template #default="{ row }">
                            <el-tag :type="statusTagType(row.status)">{{ row.status || '未处理' }}</el-tag>
                        </template>
                    </el-table-column>
                    <el-table-column label="说明" min-width="180">
                        <template #default="{ row }">
                            <span>{{ row.note || '-' }}</span>
                        </template>
                    </el-table-column>
                </el-table>
            </div>

            <div class="batch-fix-side">
                <div class="history-head">
                    <span>历史记录</span>
                    <el-button size="small" text @click="history = []">清空</el-button>
                </div>
                <div class="history-list">
                    <div v-for="item in history" :key="item.id" class="history-item">
                        <div class="history-time">{{ item.time }}</div>
                        <div class="history-text">{{ item.text }}</div>
                    </div>
                </div>
            </div>
        </div>
    </el-dialog>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useAuthStore } from '../stores/auth'
import { useUtilsStore } from '../stores/utils'
import { useUploadStore } from '../stores/upload'
import type { TemplateConfig } from '../stores/user_config'

type InputMode = 'query' | 'json'

interface CuluaItem {
    title: string
    artist: string
    collection?: string
    up?: string
    link?: string
    source?: string
    bvid?: string
    aid?: number
    cid?: number
    page?: number
    pubdate?: number
    ctime?: number
    videoDuration?: number
    partDuration?: number
    videos?: number
    videoTitle?: string
    uploader?: string
    uploaderMid?: number
}

interface BatchRow extends CuluaItem {
    selected: boolean
    originalTitle: string
    previewTitle: string
    status: string
    note: string
}

interface HistoryItem {
    id: string
    time: string
    text: string
}

const authStore = useAuthStore()
const utilsStore = useUtilsStore()
const uploadStore = useUploadStore()

const visible = ref(false)
const inputMode = ref<InputMode>('query')
const rawInput = ref('')
const correctTitle = ref('')
const correctArtist = ref('')
const delayMs = ref(1500)
const previewOnly = ref(true)
const loadingRecords = ref(false)
const previewLoading = ref(false)
const processing = ref(false)
const records = ref<BatchRow[]>([])
const history = ref<HistoryItem[]>([])
const selectedUid = ref<number | null>(null)
const progress = ref({
    done: 0,
    total: 0,
    currentKey: ''
})

const detailCache = new Map<string, TemplateConfig>()

const loginUsers = computed(() => authStore.loginUsers)
const canLoad = computed(() => rawInput.value.trim().length > 0)
const selectedCount = computed(() => records.value.filter(item => item.selected).length)
const previewReadyCount = computed(() => records.value.filter(item => !!item.previewTitle).length)
const hasPreviewableRows = computed(() => records.value.some(item => item.previewTitle))
const inputPlaceholder = computed(() =>
    inputMode.value === 'query'
        ? '输入关键词，例如：DECO 或 "さくら" "森山"'
        : '粘贴 culua 搜索返回的 JSON，支持单对象 / 数组 / { items: [...] }'
)
const progressPercent = computed(() =>
    progress.value.total > 0 ? Math.round((progress.value.done / progress.value.total) * 100) : 0
)

watch(
    loginUsers,
    users => {
        if (!selectedUid.value) {
            const user = users.find(item => !item.expired) || users[0]
            selectedUid.value = user?.uid ?? null
        }
    },
    { immediate: true }
)

function nowTime() {
    return new Date().toLocaleTimeString('zh-CN', { hour12: false })
}

function addHistory(text: string) {
    history.value.unshift({
        id: `${Date.now()}-${Math.random().toString(16).slice(2)}`,
        time: nowTime(),
        text
    })
}

function statusTagType(status: string) {
    switch (status) {
        case '预览完成':
            return 'info'
        case '已提交':
            return 'success'
        case '失败':
            return 'danger'
        case '跳过':
            return 'warning'
        default:
            return 'info'
    }
}

function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms))
}

function setAllSelected(value: boolean) {
    records.value.forEach(item => {
        item.selected = value
    })
}

function normalizeItems(payload: any): CuluaItem[] {
    if (Array.isArray(payload)) {
        return payload
    }
    if (payload && Array.isArray(payload.items)) {
        return payload.items
    }
    if (payload && typeof payload === 'object' && payload.title && payload.artist) {
        return [payload]
    }
    return []
}

function buildCore(item: CuluaItem) {
    const title = (correctTitle.value.trim() || item.title || '').trim()
    const artist = (correctArtist.value.trim() || item.artist || '').trim()

    if (title && artist) return `${title} - ${artist}`
    if (title) return title
    return artist
}

function mergeTitlePreservingContext(originalTitle: string, replacementCore: string) {
    const original = (originalTitle || '').trim()
    const suffixMatch = original.match(/(\[[^\]]+\]\s*)+$/)
    const suffix = suffixMatch?.[0] || ''
    const withoutSuffix = suffix ? original.slice(0, original.length - suffix.length) : original
    const prefixMatch = withoutSuffix.match(/^((?:\[[^\]]+\]|\d+)\.\s*)/)
    const prefix = prefixMatch?.[0] || ''
    return `${prefix}${replacementCore}${suffix}`.trim()
}

function getCacheKey(item: CuluaItem) {
    return item.aid ? `aid:${item.aid}` : `bvid:${item.bvid || ''}`
}

async function getDetailForItem(item: CuluaItem) {
    const uid = selectedUid.value
    if (!uid) {
        throw new Error('请先选择用户')
    }

    const key = getCacheKey(item)
    if (detailCache.has(key)) {
        return detailCache.get(key)!
    }

    const videoId = item.bvid || String(item.aid || '')
    if (!videoId) {
        throw new Error('缺少视频标识')
    }

    const detail = (await utilsStore.getVideoDetail(uid, videoId)) as TemplateConfig
    detailCache.set(key, detail)
    return detail
}

function findVideoIndex(detail: TemplateConfig, item: CuluaItem) {
    const videos = detail.videos || []
    const byCid = item.cid ? videos.findIndex((video: any) => Number(video.cid) === Number(item.cid)) : -1
    if (byCid >= 0) return byCid

    const pageIndex = (Number(item.page) || 1) - 1
    if (pageIndex >= 0 && pageIndex < videos.length) {
        return pageIndex
    }

    return -1
}

async function loadRecords() {
    loadingRecords.value = true
    try {
        let payload: any
        if (inputMode.value === 'query') {
            payload = await utilsStore.fetchCuluaSearch(rawInput.value.trim())
        } else {
            payload = JSON.parse(rawInput.value)
        }

        const items = normalizeItems(payload)
        records.value = items.map(item => ({
            ...item,
            selected: true,
            originalTitle: '',
            previewTitle: '',
            status: '',
            note: ''
        }))
        progress.value = { done: 0, total: 0, currentKey: '' }
        detailCache.clear()
        addHistory(`已加载 ${records.value.length} 条搜索结果`)
        utilsStore.showMessage(`已加载 ${records.value.length} 条搜索结果`, 'success')
    } catch (error) {
        console.error(error)
        utilsStore.showMessage(`读取数据失败: ${error}`, 'error')
    } finally {
        loadingRecords.value = false
    }
}

async function generatePreview() {
    if (!selectedUid.value) {
        utilsStore.showMessage('请先选择用户', 'warning')
        return
    }

    previewLoading.value = true
    try {
        for (const row of records.value) {
            row.status = ''
            row.note = ''
            row.originalTitle = ''
            row.previewTitle = ''
        }

        for (const row of records.value.filter(item => item.selected)) {
            try {
                const detail = await getDetailForItem(row)
                const index = findVideoIndex(detail, row)
                if (index < 0) {
                    row.status = '跳过'
                    row.note = '未定位到目标分P'
                    continue
                }

                const video = detail.videos[index] as any
                row.originalTitle = video.title || ''
                row.previewTitle = mergeTitlePreservingContext(row.originalTitle, buildCore(row))
                row.status = '预览完成'
                row.note = '待处理'
            } catch (error) {
                row.status = '失败'
                row.note = String(error)
            }
        }

        addHistory(`预览完成：${records.value.filter(item => item.previewTitle).length} 条记录`)
        utilsStore.showMessage(
            `预览完成：${records.value.filter(item => item.previewTitle).length} 条记录`,
            'success'
        )
    } finally {
        previewLoading.value = false
    }
}

async function runBatch() {
    if (!selectedUid.value) {
        utilsStore.showMessage('请先选择用户', 'warning')
        return
    }

    if (!records.value.some(item => item.selected && item.previewTitle)) {
        await generatePreview()
    }

    const selectedRows = records.value.filter(item => item.selected && item.previewTitle)
    if (!selectedRows.length) {
        utilsStore.showMessage('没有可处理的记录', 'warning')
        return
    }

    const groupMap = new Map<string, BatchRow[]>()
    selectedRows.forEach(row => {
        const key = getCacheKey(row)
        if (!groupMap.has(key)) groupMap.set(key, [])
        groupMap.get(key)!.push(row)
    })

    processing.value = true
    progress.value = {
        done: 0,
        total: groupMap.size,
        currentKey: ''
    }

    try {
        let index = 0
        for (const [key, rows] of groupMap.entries()) {
            progress.value.currentKey = key
            try {
                const baseDetail = await getDetailForItem(rows[0])
                const detail = JSON.parse(JSON.stringify(baseDetail)) as TemplateConfig

                for (const row of rows) {
                    const videoIndex = findVideoIndex(detail, row)
                    if (videoIndex < 0) {
                        row.status = '跳过'
                        row.note = '未定位到目标分P'
                        continue
                    }

                    detail.videos[videoIndex].title = row.previewTitle
                }

                if (previewOnly.value) {
                    rows.forEach(row => {
                        row.status = '预览完成'
                        row.note = '仅预览，未提交'
                    })
                    addHistory(`预览完成：${key}，${rows.length} 个分P`)
                } else {
                    await uploadStore.submitTemplate(selectedUid.value, detail)
                    rows.forEach(row => {
                        row.status = '已提交'
                        row.note = '编辑接口提交成功'
                    })
                    addHistory(`已提交：${key}，${rows.length} 个分P`)
                }
            } catch (error) {
                rows.forEach(row => {
                    row.status = '失败'
                    row.note = String(error)
                })
                addHistory(`处理失败：${key}，${error}`)
            }

            index += 1
            progress.value.done = index

            if (!previewOnly.value && index < groupMap.size && delayMs.value > 0) {
                await sleep(delayMs.value)
            }
        }

        utilsStore.showMessage(
            previewOnly.value
                ? `预览完成：${selectedRows.length} 条记录`
                : `批处理完成：${selectedRows.length} 条记录`,
            'success'
        )
    } finally {
        processing.value = false
        progress.value.currentKey = ''
    }
}
</script>

<style scoped>
.batch-fix-layout {
    display: grid;
    grid-template-columns: minmax(0, 1fr) 280px;
    gap: 16px;
}

.batch-fix-main {
    min-width: 0;
}

.batch-fix-toolbar {
    display: flex;
    gap: 10px;
    align-items: center;
    flex-wrap: wrap;
    margin-bottom: 12px;
}

.second-row :deep(.el-input) {
    width: 280px;
}

.tool-user-select {
    width: 260px;
}

.delay-label,
.summary-text {
    color: #606266;
    font-size: 13px;
}

.progress-block {
    margin-bottom: 12px;
}

.progress-text {
    margin-top: 6px;
    color: #606266;
    font-size: 13px;
}

.batch-fix-side {
    border-left: 1px solid #ebeef5;
    padding-left: 16px;
    display: flex;
    flex-direction: column;
    min-height: 0;
}

.history-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 10px;
    font-weight: 600;
}

.history-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
    max-height: 640px;
    overflow: auto;
}

.history-item {
    padding: 10px 12px;
    border-radius: 8px;
    background: #f5f7fa;
}

.history-time {
    color: #909399;
    font-size: 12px;
    margin-bottom: 4px;
}

.history-text {
    color: #303133;
    font-size: 13px;
    word-break: break-word;
}
</style>
