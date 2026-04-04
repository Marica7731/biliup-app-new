import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { useUtilsStore } from './utils'

interface UploadTask {
    id: string
    template: string
    user: any
    video: any
    status: string
    progress: number
    total_transmit_bytes: number
    speed: number
    total_size: number
    error_message?: string
    created_at: number
    started_at?: number
    finished_at?: number
    retry_count: number
}

interface UploadRuntimeStatus {
    cooldown_until_ms?: number | null
    cooldown_secs: number
}

export const useUploadStore = defineStore('upload', () => {
    const uploadQueue = ref<UploadTask[]>([])
    const uploadRuntimeStatus = ref<UploadRuntimeStatus>({
        cooldown_until_ms: null,
        cooldown_secs: 300
    })
    const utilsStore = useUtilsStore()
    const genericRetryScheduled = new Set<string>()
    const stalledRetryCooldown = new Set<string>()

    const isRateLimitError = (errorMessage?: string) => {
        const msg = (errorMessage || '').toLowerCase()
        return msg.includes('code: 601') || msg.includes('upload rate limit') || msg.includes('上传视频过快')
    }

    const cleanupRetryState = (queue: UploadTask[]) => {
        const alive = new Set(queue.map(task => task.id))
        for (const id of Array.from(genericRetryScheduled)) {
            if (!alive.has(id)) {
                genericRetryScheduled.delete(id)
            }
        }
        for (const id of Array.from(stalledRetryCooldown)) {
            if (!alive.has(id)) {
                stalledRetryCooldown.delete(id)
            }
        }
    }

    const scheduleGenericRetry = (task: UploadTask) => {
        if (genericRetryScheduled.has(task.id)) return
        if ((task.retry_count || 0) >= 3) return
        genericRetryScheduled.add(task.id)
        setTimeout(async () => {
            try {
                const latestTask = uploadQueue.value.find(item => item.id === task.id)
                if (!latestTask || latestTask.status !== 'Failed') {
                    return
                }
                if (isRateLimitError(latestTask.error_message)) {
                    return
                }
                await retryUpload(task.id, false)
                utilsStore.showMessage(`${task.user.username}-${task.video.title} 自动重试中`)
            } catch (error) {
                console.error('通用失败自动重试失败:', error)
            } finally {
                genericRetryScheduled.delete(task.id)
            }
        }, 30000)
    }

    const getUploadRuntimeStatus = async () => {
        try {
            const status: UploadRuntimeStatus = await invoke('get_upload_runtime_status')
            uploadRuntimeStatus.value = status
            return status
        } catch (error) {
            console.error('获取上传运行状态失败:', error)
            throw error
        }
    }

    // 创建上传任务
    const createUploadTask = async (uid: number, template: string, videoFiles: any[]) => {
        try {
            var count = 0
            // 遍历videoFiles, 将所有id不存在uploadQueue中的任务添加到uploadQueue
            for (const video of videoFiles) {
                console.log(video)
                if (video.complete !== undefined && video.complete) {
                    console.log('跳过已完成视频文件:', video)
                    continue
                }
                if (!uploadQueue.value.some(task => task.id === video.id)) {
                    try {
                        const normalizedVideo = {
                            ...video,
                            // 重新入队时清理旧上传态，避免携带历史cid导致提交异常（如21015）
                            cid: 0,
                            complete: false,
                            status: 'Waiting',
                            errorMessage: '',
                            progress: 0,
                            finished_at: 0
                        }
                        await invoke('create_upload_task', {
                            uid,
                            template,
                            video: normalizedVideo
                        })
                        count++
                    } catch (error) {
                        console.error('创建上传任务失败:', error)
                        throw error
                    }
                }
            }

            await getUploadQueue() // 刷新队列
            return count
        } catch (error) {
            console.error('创建上传任务失败:', error)
            throw error
        }
    }

    // 开始上传
    const startUpload = async (taskId: string) => {
        try {
            const success = await invoke('start_upload', { taskId })
            await getUploadQueue()

            return success ? 1 : 0
        } catch (error) {
            console.error('开始上传失败:', error)
            throw error
        }
    }

    // 暂停上传
    const pauseUpload = async (taskId: string) => {
        try {
            const paused = await invoke('pause_upload', { taskId })
            await getUploadQueue()

            return paused ? 1 : 0
        } catch (error) {
            console.error('暂停上传失败:', error)
            throw error
        }
    }

    // 取消上传
    const cancelUpload = async (taskId: string) => {
        try {
            const canceled = await invoke('cancel_upload', { taskId })
            await getUploadQueue()

            return canceled ? 1 : 0
        } catch (error) {
            console.error('取消上传失败:', error)
            throw error
        }
    }

    // 获取上传队列
    const getUploadQueue = async () => {
        try {
            const queue: UploadTask[] = await invoke('get_upload_queue')
            uploadQueue.value = queue
            await getUploadRuntimeStatus()
            // 更新速率
            const now = Date.now()
            queue
                .filter(task => task.started_at && task.status === 'Running')
                .forEach(task => {
                    const elapsed = now - task.started_at!
                    task.speed = elapsed > 0 ? (task.total_transmit_bytes * 1000) / elapsed : 0
                })

            queue
                .filter(
                    task =>
                        task.started_at &&
                        task.status === 'Running' &&
                        task.speed === 0 &&
                        now - task.started_at! > 30000
                )
                .forEach(task => {
                    if (stalledRetryCooldown.has(task.id)) {
                        return
                    }
                    stalledRetryCooldown.add(task.id)
                    if (task.retry_count && task.retry_count >= 3) {
                        utilsStore.showMessage(
                            `${task.user.username}-${task.video.title} 超过 3 次重试，取消任务`
                        )
                        cancelUpload(task.id)
                    } else {
                        utilsStore.showMessage(
                            `${task.user.username}-${task.video.title} 超过 30 秒未上传，正在重试...`
                        )
                        retryUpload(task.id, false)
                    }
                    setTimeout(() => {
                        stalledRetryCooldown.delete(task.id)
                    }, 10000)
                })

            const failedTasks = queue.filter(task => task.status === 'Failed')

            failedTasks
                .filter(task => !isRateLimitError(task.error_message))
                .forEach(task => {
                    scheduleGenericRetry(task)
                })

            cleanupRetryState(queue)

            return queue
        } catch (error) {
            console.error('获取上传队列失败:', error)
            throw error
        }
    }

    // 重试上传
    const retryUpload = async (taskId: string, refresh: boolean = true) => {
        try {
            await invoke('retry_upload', { taskId })
            if (refresh) {
                await getUploadQueue()
            }
        } catch (error) {
            console.error('重试上传失败:', error)
            throw error
        }
    }

    // 提交视频
    const submitTemplate = async (uid: number, upload: any) => {
        try {
            const result = await invoke('submit', { uid, form: upload })
            return result
        } catch (error) {
            console.error('提交视频失败:', error)
            throw error
        }
    }

    const getUploadTask = (taskId: string) => {
        const task = uploadQueue.value.find(t => t.id === taskId)
        if (task) {
            return task
        } else {
            return null
        }
    }

    return {
        uploadQueue,
        createUploadTask,
        startUpload,
        pauseUpload,
        cancelUpload,
        getUploadQueue,
        uploadRuntimeStatus,
        retryUpload,
        submitTemplate,
        getUploadTask,
        getUploadRuntimeStatus
    }
})
