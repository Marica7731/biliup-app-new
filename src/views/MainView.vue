<template>
    <div class="tool-shell">
        <header class="hero">
            <div>
                <h1>Culua Batch Editor</h1>
                <p>本地私有工具。用于读取 culua 搜索结果并批量修正 B 站稿件分P标题。</p>
            </div>
            <div class="hero-actions">
                <el-button type="primary" @click="showLoginDialog = true">登录用户</el-button>
                <BatchFixTitlesDialog v-if="loginUsers.length > 0" />
            </div>
        </header>

        <section class="panel">
            <div class="panel-head">
                <h2>已登录用户</h2>
                <div class="panel-actions">
                    <el-button size="small" @click="refreshUsers" :loading="loadingUsers">刷新</el-button>
                </div>
            </div>

            <div v-if="loadingUsers" class="empty-state">正在读取登录状态...</div>

            <div v-else-if="loginUsers.length === 0" class="empty-state">
                <p>当前没有可用登录用户。</p>
                <p>先登录一个能编辑目标稿件的 B 站账号，再执行批处理。</p>
            </div>

            <div v-else class="user-list">
                <div v-for="user in loginUsers" :key="user.uid" class="user-card">
                    <div class="user-main">
                        <el-avatar
                            :src="user.expired ? '' : `data:image/jpeg;base64,${user.avatar}`"
                            :size="44"
                        >
                            {{ user.username.charAt(0) }}
                        </el-avatar>
                        <div>
                            <div class="user-name-row">
                                <span class="user-name">{{ user.username }}</span>
                                <el-tag :type="user.expired ? 'danger' : 'success'">
                                    {{ user.expired ? 'Cookie失效' : '可用' }}
                                </el-tag>
                            </div>
                            <div class="user-meta">UID: {{ user.uid }}</div>
                        </div>
                    </div>
                    <el-button size="small" type="danger" plain @click="logout(user.uid)">登出</el-button>
                </div>
            </div>
        </section>

        <section class="panel">
            <div class="panel-head">
                <h2>使用说明</h2>
            </div>
            <ol class="steps">
                <li>登录目标 B 站账号。</li>
                <li>点击右上角“批量修正分P”。</li>
                <li>输入关键词或粘贴 culua JSON。</li>
                <li>填写正确标题或正确歌手。</li>
                <li>先勾选“仅预览，不提交”检查结果。</li>
                <li>确认后取消预览并执行批处理。</li>
            </ol>
        </section>

        <el-dialog
            v-model="showLoginDialog"
            title="登录用户"
            width="760px"
            append-to-body
            destroy-on-close
        >
            <LoginView @login-success="handleLoginSuccess" />
        </el-dialog>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { ElMessageBox } from 'element-plus'
import { useAuthStore } from '../stores/auth'
import { useUtilsStore } from '../stores/utils'
import LoginView from '../components/LoginView.vue'
import BatchFixTitlesDialog from '../components/BatchFixTitlesDialog.vue'

const authStore = useAuthStore()
const utilsStore = useUtilsStore()

const showLoginDialog = ref(false)
const loadingUsers = ref(false)

const loginUsers = computed(() => authStore.loginUsers)

async function refreshUsers() {
    loadingUsers.value = true
    try {
        await authStore.getLoginUsers()
    } catch (error) {
        utilsStore.showMessage(`读取登录用户失败: ${error}`, 'error')
    } finally {
        loadingUsers.value = false
    }
}

async function handleLoginSuccess() {
    showLoginDialog.value = false
    await refreshUsers()
    utilsStore.showMessage('登录成功', 'success')
}

async function logout(uid: number) {
    await ElMessageBox.confirm('确认登出该用户？', '提示', {
        type: 'warning'
    })

    try {
        await authStore.logoutUser(uid)
        await refreshUsers()
        utilsStore.showMessage('已登出用户', 'success')
    } catch (error) {
        utilsStore.showMessage(`登出失败: ${error}`, 'error')
    }
}

onMounted(async () => {
    await refreshUsers()
})
</script>

<style scoped>
.tool-shell {
    min-height: 100vh;
    padding: 24px;
    background: linear-gradient(180deg, #f4f7fb 0%, #eef2f7 100%);
    color: #1f2937;
}

.hero,
.panel {
    max-width: 1180px;
    margin: 0 auto 18px;
    background: #fff;
    border: 1px solid #e5e7eb;
    border-radius: 16px;
    box-shadow: 0 10px 30px rgba(15, 23, 42, 0.06);
}

.hero {
    padding: 24px 28px;
    display: flex;
    justify-content: space-between;
    gap: 16px;
    align-items: flex-start;
}

.hero h1 {
    font-size: 28px;
    line-height: 1.2;
    margin-bottom: 8px;
}

.hero p {
    color: #4b5563;
    max-width: 760px;
}

.hero-actions {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
    align-items: center;
}

.panel {
    padding: 20px 24px;
}

.panel-head {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
}

.panel-head h2 {
    font-size: 18px;
}

.empty-state {
    color: #6b7280;
    line-height: 1.8;
}

.user-list {
    display: grid;
    gap: 12px;
}

.user-card {
    border: 1px solid #e5e7eb;
    border-radius: 12px;
    padding: 14px 16px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 16px;
}

.user-main {
    display: flex;
    gap: 12px;
    align-items: center;
}

.user-name-row {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 6px;
}

.user-name {
    font-weight: 600;
}

.user-meta {
    color: #6b7280;
    font-size: 13px;
}

.steps {
    padding-left: 18px;
    line-height: 1.9;
    color: #374151;
}

@media (max-width: 900px) {
    .tool-shell {
        padding: 16px;
    }

    .hero {
        flex-direction: column;
    }

    .user-card {
        flex-direction: column;
        align-items: flex-start;
    }
}
</style>
