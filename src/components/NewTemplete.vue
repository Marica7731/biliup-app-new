<template>
    <div>
        <!-- 创建模板对话框 -->
        <el-dialog v-model="showDialog" title="新建模板" width="620px">
            <el-form :model="newTemplateForm" label-width="98px">
                <el-form-item label="选择用户">
                    <el-select v-model="newTemplateForm.userUid" placeholder="请选择用户">
                        <el-option
                            v-for="user in loginUsers"
                            :key="user.uid"
                            :label="user.username"
                            :value="user.uid"
                        >
                            <div class="user-option">
                                <el-avatar
                                    :src="`data:image/jpeg;base64,${user.avatar}`"
                                    :size="20"
                                >
                                    {{ user.username.charAt(0) }}
                                </el-avatar>
                                <span class="user-option-name">{{ user.username }}</span>
                            </div>
                        </el-option>
                    </el-select>
                </el-form-item>

                <el-form-item label="模板类型">
                    <el-radio-group v-model="newTemplateForm.templateType">
                        <el-radio value="blank">空白模板</el-radio>
                        <el-radio value="bv">BV/AV号</el-radio>
                    </el-radio-group>
                </el-form-item>

                <el-form-item
                    label="模板名称"
                    required
                    v-if="newTemplateForm.templateType === 'blank'"
                >
                    <el-input
                        v-model="newTemplateForm.name"
                        placeholder="请输入模板名称"
                        maxlength="50"
                    />
                </el-form-item>

                <el-form-item label="BV/AV号" required v-if="newTemplateForm.templateType === 'bv'">
                    <el-input
                        v-model="newTemplateForm.bvNumber"
                        placeholder="请输入BV号或AV号，如: BV1xx4y1z7xx 或 av12345"
                        maxlength="20"
                    />
                </el-form-item>

                <el-form-item label="操作类型" v-if="newTemplateForm.templateType === 'bv'">
                    <el-radio-group v-model="newTemplateForm.actionType">
                        <el-radio value="edit">编辑</el-radio>
                        <el-radio value="copy">复制</el-radio>
                    </el-radio-group>
                    <div class="form-tip">
                        <div>编辑：直接修改现有稿件</div>
                        <div>复制：基于现有稿件创建新模板</div>
                    </div>
                </el-form-item>
                <el-form-item
                    label="模板名称"
                    required
                    v-if="
                        newTemplateForm.templateType === 'bv' &&
                        newTemplateForm.actionType === 'copy'
                    "
                >
                    <el-input
                        v-model="newTemplateForm.name"
                        placeholder="请输入模板名称"
                        maxlength="50"
                    />
                </el-form-item>

                <el-form-item label="模板文件夹" v-if="newTemplateForm.templateType === 'blank'">
                    <div class="folder-row">
                        <el-input
                            v-model="newTemplateForm.folderPath"
                            placeholder="可选：选择包含标题/简介/标签/封面的文件夹"
                            readonly
                        />
                        <el-button @click="pickContentFolder">选择文件夹</el-button>
                        <el-button text @click="clearContentFolder">清除</el-button>
                    </div>
                    <div class="form-tip">创建后仅应用到当前标签页草稿（不自动保存模板）</div>
                </el-form-item>

                <el-form-item
                    label="应用字段"
                    v-if="newTemplateForm.templateType === 'blank' && newTemplateForm.folderPath"
                >
                    <el-checkbox-group v-model="newTemplateForm.applyFields">
                        <el-checkbox value="cover">封面</el-checkbox>
                        <el-checkbox value="desc">简介</el-checkbox>
                        <el-checkbox value="tag">标签</el-checkbox>
                        <el-checkbox value="source">来源</el-checkbox>
                        <el-checkbox value="title">标题</el-checkbox>
                    </el-checkbox-group>
                </el-form-item>
            </el-form>
            <template #footer>
                <span class="dialog-footer">
                    <el-button @click="closeDialog">取消</el-button>
                    <el-button type="primary" @click="createNewTemplate">确定</el-button>
                </span>
            </template>
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { useAuthStore } from '../stores/auth'
import { useUserConfigStore } from '../stores/user_config'
import { useUtilsStore } from '../stores/utils'

// Props
const props = defineProps<{
    modelValue: boolean
}>()

// Emits
const emit = defineEmits<{
    'update:modelValue': [value: boolean]
    'template-created': [userUid: number, templateName: string]
    'apply-folder-to-template': [
        userUid: number,
        templateName: string,
        folderPath: string,
        options: { title: boolean; cover: boolean; desc: boolean; tag: boolean; source: boolean }
    ]
}>()

// Stores
const authStore = useAuthStore()
const userConfigStore = useUserConfigStore()
const utilsStore = useUtilsStore()

// 计算属性
const loginUsers = computed(() => authStore.loginUsers)

// 响应式数据
const showDialog = computed({
    get: () => props.modelValue,
    set: value => emit('update:modelValue', value)
})

const DEFAULT_TEMPLATE_CONTENT_DIR_KEY = 'default-template-content-dir-new-template'
const NEW_TEMPLATE_APPLY_FIELDS_KEY = 'new-template-apply-fields'

const newTemplateForm = ref({
    userUid: null,
    name: '',
    templateType: 'blank', // 'blank' | 'bv'
    bvNumber: '',
    actionType: 'copy', // 'edit' | 'copy'
    folderPath: '',
    applyFields: ['cover', 'desc', 'tag', 'source', 'title'] as string[]
})

// localStorage key for storing user preferences
const STORAGE_KEY = 'new-template-preferences'

// 加载用户上次的选择
const loadUserPreferences = () => {
    try {
        const saved = localStorage.getItem(STORAGE_KEY)
        if (saved) {
            const preferences = JSON.parse(saved)
            // 只加载有效的用户UID（确保用户仍然登录）
            if (
                preferences.userUid &&
                loginUsers.value.some(user => user.uid === preferences.userUid)
            ) {
                newTemplateForm.value.userUid = preferences.userUid
            }
            // 加载其他设置
            if (preferences.templateType) {
                newTemplateForm.value.templateType = preferences.templateType
            }
            if (preferences.actionType) {
                newTemplateForm.value.actionType = preferences.actionType
            }
            if (Array.isArray(preferences.applyFields)) {
                newTemplateForm.value.applyFields = preferences.applyFields.filter((v: string) =>
                    ['cover', 'desc', 'tag', 'source', 'title'].includes(v)
                )
            }
        }
    } catch (error) {
        console.error('加载用户偏好设置失败:', error)
    }

    try {
        const rawFields = localStorage.getItem(NEW_TEMPLATE_APPLY_FIELDS_KEY)
        if (rawFields) {
            const fields = JSON.parse(rawFields)
            if (Array.isArray(fields)) {
                newTemplateForm.value.applyFields = fields.filter((v: string) =>
                    ['cover', 'desc', 'tag', 'source', 'title'].includes(v)
                )
            }
        }
    } catch (error) {
        console.error('加载应用字段偏好失败:', error)
    }
}

// 保存用户选择
const saveUserPreferences = () => {
    try {
        const preferences = {
            userUid: newTemplateForm.value.userUid,
            templateType: newTemplateForm.value.templateType,
            actionType: newTemplateForm.value.actionType,
            applyFields: newTemplateForm.value.applyFields
        }
        localStorage.setItem(STORAGE_KEY, JSON.stringify(preferences))
        localStorage.setItem(
            NEW_TEMPLATE_APPLY_FIELDS_KEY,
            JSON.stringify(newTemplateForm.value.applyFields || [])
        )
    } catch (error) {
        console.error('保存用户偏好设置失败:', error)
    }
}

// 监听表单变化，自动保存用户偏好
watch(() => newTemplateForm.value.userUid, saveUserPreferences)
watch(() => newTemplateForm.value.templateType, saveUserPreferences)
watch(() => newTemplateForm.value.actionType, saveUserPreferences)
watch(
    () => newTemplateForm.value.applyFields,
    () => saveUserPreferences(),
    { deep: true }
)

// 当对话框打开时加载用户偏好
watch(
    () => props.modelValue,
    newValue => {
        if (newValue) {
            loadUserPreferences()
        }
    }
)

// 方法
const closeDialog = () => {
    showDialog.value = false
    resetForm()
}

const resetForm = () => {
    // 只重置输入字段，保留用户偏好设置
    newTemplateForm.value.name = ''
    newTemplateForm.value.bvNumber = ''
    newTemplateForm.value.folderPath = ''
    // 不重置 userUid, templateType, actionType，保持用户上次的选择
}

const pickContentFolder = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: localStorage.getItem(DEFAULT_TEMPLATE_CONTENT_DIR_KEY) || undefined
        })
        if (!selected) return
        const picked = String(selected)
        newTemplateForm.value.folderPath = picked
        localStorage.setItem(DEFAULT_TEMPLATE_CONTENT_DIR_KEY, picked)
    } catch (error) {
        console.error('选择文件夹失败:', error)
        utilsStore.showMessage(`选择文件夹失败: ${error}`, 'error')
    }
}

const clearContentFolder = () => {
    newTemplateForm.value.folderPath = ''
}

// 创建新模板
const createNewTemplate = async () => {
    const targetUserUid = newTemplateForm.value.userUid
    if (!targetUserUid) {
        utilsStore.showMessage('请选择用户', 'error')
        return
    }

    try {
        if (newTemplateForm.value.templateType === 'blank') {
            // 空白模板
            if (!newTemplateForm.value.name.trim()) {
                utilsStore.showMessage('请输入模板名称', 'error')
                return
            }

            const templateName = newTemplateForm.value.name.trim()
            await userConfigStore.addUserTemplate(targetUserUid, templateName)

            utilsStore.showMessage('空白模板创建成功', 'success')
            emit('template-created', targetUserUid, templateName)
            if (newTemplateForm.value.folderPath) {
                const opts = new Set(newTemplateForm.value.applyFields || [])
                emit('apply-folder-to-template', targetUserUid, templateName, newTemplateForm.value.folderPath, {
                    title: opts.has('title'),
                    cover: opts.has('cover'),
                    desc: opts.has('desc'),
                    tag: opts.has('tag'),
                    source: opts.has('source')
                })
            }
        } else if (newTemplateForm.value.templateType === 'bv') {
            // BV/AV号模板
            if (!newTemplateForm.value.bvNumber.trim()) {
                utilsStore.showMessage('请输入BV号或AV号', 'error')
                return
            }

            const bvNumber = newTemplateForm.value.bvNumber.trim()
            const actionType = newTemplateForm.value.actionType
            const templateName = newTemplateForm.value.name.trim() || `${bvNumber}`

            const isEdit = actionType === 'edit'
            await createTemplateFromBV(targetUserUid, bvNumber, templateName, isEdit)

            utilsStore.showMessage('基于稿件创建模板成功', 'success')
            emit('template-created', targetUserUid, templateName)
            if (newTemplateForm.value.folderPath) {
                const opts = new Set(newTemplateForm.value.applyFields || [])
                emit('apply-folder-to-template', targetUserUid, templateName, newTemplateForm.value.folderPath, {
                    title: opts.has('title'),
                    cover: opts.has('cover'),
                    desc: opts.has('desc'),
                    tag: opts.has('tag'),
                    source: opts.has('source')
                })
            }
        }

        closeDialog()
    } catch (error) {
        console.error('创建模板失败:', error)
        // 提供更详细的错误信息
        let errorMessage = '创建模板失败'
        if (error instanceof Error) {
            errorMessage = `创建模板失败: ${error.message}`
        } else if (typeof error === 'string') {
            errorMessage = `创建模板失败: ${error}`
        }
        utilsStore.showMessage(errorMessage, 'error')
    }
}

// 根据BV号创建模板
const createTemplateFromBV = async (
    userUid: number,
    bvNumber: string,
    templateName: string,
    isEdit: boolean
) => {
    try {
        console.log(`从BV号 ${bvNumber} 创建模板: ${templateName}`)

        const newTemplate = (await utilsStore.getVideoDetail(userUid, bvNumber)) as any

        // 处理视频列表
        if (newTemplate.videos && Array.isArray(newTemplate.videos)) {
            for (const video of newTemplate.videos) {
                video.id = video.filename
                video.path = ''
            }
        }

        if (newTemplate.aid && (await utilsStore.getSeasonList(userUid))) {
            const season_id = await utilsStore.getVideoSeason(userUid, newTemplate.aid)

            if (season_id !== 0) {
                const section_id = await utilsStore.seasonlist.find(
                    item => item.season_id === season_id
                )?.section_id
                newTemplate.season_id = season_id
                newTemplate.section_id = section_id
            }
        }

        if (!isEdit) {
            newTemplate.aid = undefined
        }

        await userConfigStore.addUserTemplate(userUid, templateName, newTemplate)
        console.log(
            `BV模板添加完成: uid=${userUid}, template=${templateName}, videos=${
                Array.isArray(newTemplate.videos) ? newTemplate.videos.length : 0
            }`
        )
    } catch (error) {
        console.error('从BV号创建模板失败: ', error)
        utilsStore.showMessage(`从BV号创建模板失败: ${error}`, 'error')
        throw error
    }
}

// 暴露函数供父组件调用
defineExpose({
    createTemplateFromBV
})
</script>

<style scoped>
.user-option {
    display: flex;
    align-items: center;
    gap: 8px;
}

.user-option-name {
    font-size: 14px;
}

.form-tip {
    margin-top: 8px;
    font-size: 12px;
    color: #666;
    line-height: 1.4;
}

.form-tip > div {
    margin-bottom: 2px;
}

.dialog-footer {
    display: flex;
    gap: 10px;
}

.folder-row {
    display: grid;
    grid-template-columns: minmax(0, 1fr) auto auto;
    align-items: center;
    gap: 8px;
    width: 100%;
}

.folder-row :deep(.el-input) {
    min-width: 0;
}

.folder-row :deep(.el-button) {
    white-space: nowrap;
}

:deep(.el-form-item__label) {
    white-space: nowrap;
}
</style>
