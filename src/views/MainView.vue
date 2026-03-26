<template>
    <div class="main-view">
        <!-- 拖拽覆盖层 -->
        <div v-if="isDragOver" class="drag-overlay">
            <div class="drag-content">
                <el-icon class="drag-icon"><upload-filled /></el-icon>
                <h3>拖拽视频文件到此处</h3>
                <p>支持 MP4、AVI、MOV、MKV、WMV、FLV、M4V、WEBM 格式</p>
                <p v-if="!selectedUser || !currentTemplateName" class="warning-text">
                    请先选择用户和模板
                </p>
            </div>
        </div>

        <!-- 顶部导航栏 -->
        <el-header class="header">
            <div class="header-content">
                <div class="header-left">
                    <h2 class="app-title">Biliup APP</h2>
                    <div class="app-version">(v{{ currentVer }})</div>
                </div>
                <div class="header-center">
                    <el-button type="info" size="small" @click="exportLogs" title="导出日志">
                        导出日志
                    </el-button>
                    <el-button type="primary" size="small" @click="checkUpdate" title="检查更新">
                        检查更新
                    </el-button>
                </div>
                <div class="header-right">
                    <!-- 上传队列下拉框 -->
                    <UploadQueue />

                    <!-- 全局设置按钮 -->
                    <el-button
                        type="info"
                        size="small"
                        circle
                        @click="showGlobalConfigDialog = true"
                        title="全局设置"
                        class="global-config-btn"
                    >
                        <el-icon><setting /></el-icon>
                    </el-button>

                    <!-- 用户列表下拉框 -->
                    <UserList
                        @show-login="showLoginDialog = true"
                        @user-logout="handleLogoutUser"
                    />
                </div>
            </div>
        </el-header>

        <el-container class="main-container">
            <!-- 用户模板侧边栏 -->
            <el-aside ref="sidebarRef" class="sidebar" :style="sidebarInlineStyle">
                <div class="sidebar-header">
                    <h3></h3>
                    <div class="header-buttons">
                        <el-checkbox
                            v-model="highlightAutoSubmitting"
                            size="small"
                            class="highlight-checkbox"
                        >
                            <span class="highlight-checkbox-text"> 高亮显示<br />自动提交 </span>
                        </el-checkbox>
                        <el-button
                            type="success"
                            size="small"
                            @click="showLoginDialog = true"
                            :disabled="templateLoading"
                        >
                            <el-icon><user /></el-icon>
                            登录用户
                        </el-button>
                        <el-button
                            type="primary"
                            size="small"
                            @click="showNewTemplateDialog = true"
                            :disabled="!loginUsers.length || templateLoading"
                        >
                            <el-icon><plus /></el-icon>
                            新建模板
                        </el-button>
                    </div>
                </div>

                <div class="sidebar-content">
                    <div class="user-template-list">
                        <div
                            v-for="userTemplate in userTemplates"
                            :key="userTemplate.user.uid"
                            class="user-section"
                        >
                            <!-- 用户头部 -->
                            <div
                                class="user-header"
                                @click="handleUserExpansion(userTemplate.user.uid)"
                                :class="{
                                    disabled: templateLoading || userTemplate.user.expired,
                                    'user-header-expired': userTemplate.user.expired
                                }"
                            >
                                <el-avatar
                                    :src="
                                        userTemplate.user.expired
                                            ? ''
                                            : `data:image/jpeg;base64,${userTemplate.user.avatar}`
                                    "
                                    :size="24"
                                    class="user-avatar"
                                    :class="{ 'user-avatar-expired': userTemplate.user.expired }"
                                >
                                    {{ userTemplate.user.username.charAt(0) }}
                                </el-avatar>
                                <span class="user-name">{{ userTemplate.user.username }}</span>
                                <el-icon
                                    class="config-icon"
                                    :class="{ disabled: userTemplate.user.expired }"
                                    @click.stop="openUserConfig(userTemplate.user)"
                                    title="用户配置"
                                >
                                    <setting />
                                </el-icon>
                                <el-badge
                                    :value="userTemplate.templates.length"
                                    class="template-count-badge"
                                />
                                <el-icon
                                    class="expand-icon"
                                    :class="{ expanded: userTemplate.expanded }"
                                >
                                    <arrow-down />
                                </el-icon>
                            </div>

                            <!-- 模板列表 -->
                            <div class="template-list" v-show="userTemplate.expanded">
                                <div class="template-tools">
                                    <el-select
                                        size="small"
                                        class="template-sort-select"
                                        :model-value="getTemplateSortMode(userTemplate.user.uid)"
                                        :disabled="templateLoading || userTemplate.user.expired"
                                        @change="
                                            (mode: string) =>
                                                updateTemplateSortMode(userTemplate.user.uid, mode)
                                        "
                                    >
                                        <el-option label="手动排序" value="manual" />
                                        <el-option label="A-Z 排序" value="az" />
                                    </el-select>
                                    <el-button
                                        size="small"
                                        link
                                        :disabled="templateLoading || userTemplate.user.expired"
                                        @click.stop="createTemplateFolder(userTemplate.user.uid)"
                                    >
                                        新建模板夹
                                    </el-button>
                                </div>
                                <div
                                    v-for="item in getTemplateRenderItems(
                                        userTemplate.user.uid,
                                        userTemplate.templates
                                    )"
                                    :key="`${userTemplate.user.uid}-${item.id}`"
                                    class="template-entry"
                                    :class="{
                                        'entry-folder': item.kind === 'folder',
                                        'entry-template': item.kind === 'template'
                                    }"
                                >
                                    <div
                                        v-if="item.kind === 'folder'"
                                        class="template-folder-header"
                                        @click="
                                            toggleFolderCollapsed(
                                                userTemplate.user.uid,
                                                item.folderId
                                            )
                                        "
                                    >
                                        <div class="template-folder-info">
                                            <img
                                                class="template-folder-cover"
                                                :src="
                                                    resolveCoverSrc(
                                                        getFolderCoverPath(
                                                            userTemplate.user.uid,
                                                            item.folderId
                                                        ),
                                                        'folder'
                                                    )
                                                "
                                                @error="
                                                    event =>
                                                        handleCoverError(
                                                            event,
                                                            getFolderCoverPath(
                                                                userTemplate.user.uid,
                                                                item.folderId
                                                            ),
                                                            'folder'
                                                        )
                                                "
                                            />
                                            <span class="template-folder-title">
                                                {{ item.name }}
                                                <small>({{ item.count }})</small>
                                            </span>
                                        </div>
                                        <div class="template-folder-actions" @click.stop>
                                            <el-icon
                                                class="folder-toggle-icon"
                                                :class="{
                                                    collapsed: isFolderCollapsed(
                                                        userTemplate.user.uid,
                                                        item.folderId
                                                    )
                                                }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                            <el-dropdown
                                                trigger="click"
                                                :disabled="templateLoading || userTemplate.user.expired"
                                            >
                                                <el-button link size="small">管理</el-button>
                                                <template #dropdown>
                                                    <el-dropdown-menu>
                                                        <el-dropdown-item
                                                            @click.stop="
                                                                selectFolderCover(
                                                                    userTemplate.user.uid,
                                                                    item.folderId
                                                                )
                                                            "
                                                            >设置封面</el-dropdown-item
                                                        >
                                                        <el-dropdown-item
                                                            @click.stop="
                                                                clearFolderCoverPath(
                                                                    userTemplate.user.uid,
                                                                    item.folderId
                                                                )
                                                            "
                                                            >清除封面</el-dropdown-item
                                                        >
                                                        <el-dropdown-item
                                                            @click.stop="
                                                                renameTemplateFolder(
                                                                    userTemplate.user.uid,
                                                                    item.folderId
                                                                )
                                                            "
                                                            >重命名</el-dropdown-item
                                                        >
                                                        <el-dropdown-item
                                                            divided
                                                            @click.stop="
                                                                deleteTemplateFolder(
                                                                    userTemplate.user.uid,
                                                                    item.folderId
                                                                )
                                                            "
                                                            >删除</el-dropdown-item
                                                        >
                                                    </el-dropdown-menu>
                                                </template>
                                            </el-dropdown>
                                        </div>
                                    </div>
                                    <div
                                        v-else
                                        class="template-item"
                                        :class="{
                                            active:
                                                selectedUser?.uid === userTemplate.user.uid &&
                                                currentTemplateName === item.template.name,
                                            'auto-submitting':
                                                highlightAutoSubmitting &&
                                                isTemplateAutoSubmitting(
                                                    userTemplate.user.uid,
                                                    item.template.name
                                                ),
                                            'auto-submitting-simple':
                                                !highlightAutoSubmitting &&
                                                isTemplateAutoSubmitting(
                                                    userTemplate.user.uid,
                                                    item.template.name
                                                ),
                                            'template-loading':
                                                templateLoading &&
                                                selectedUser?.uid === userTemplate.user.uid &&
                                                currentTemplateName === item.template.name,
                                            disabled: templateLoading || userTemplate.user.expired,
                                            'template-item-in-folder': item.folderId
                                        }"
                                        @click="
                                            handleTemplateSelection(
                                                userTemplate.user,
                                                item.template.name,
                                                $event
                                            )
                                        "
                                        :draggable="
                                            isTemplateDragEnabled(
                                                userTemplate.user.uid,
                                                userTemplate.user.expired
                                            )
                                        "
                                        @dragstart="
                                            onTemplateDragStart(
                                                $event,
                                                userTemplate.user.uid,
                                                item.template.name,
                                                item.folderId
                                            )
                                        "
                                        @dragend="onTemplateDragEnd"
                                        @dragover.prevent="
                                            onTemplateDragOver(
                                                $event,
                                                userTemplate.user.uid,
                                                item.template.name,
                                                item.folderId
                                            )
                                        "
                                        @drop.prevent="
                                            onTemplateDrop(
                                                $event,
                                                userTemplate.user.uid,
                                                item.template.name,
                                                item.folderId
                                            )
                                        "
                                    >
                                        <img
                                            class="template-cover"
                                            :style="templateCoverStyle"
                                            :src="
                                                resolveCoverSrc(
                                                    getTemplateCoverPath(
                                                        userTemplate.user.uid,
                                                        item.template.name
                                                    ),
                                                    'template'
                                                )
                                            "
                                            @error="
                                                event =>
                                                    handleCoverError(
                                                        event,
                                                        getTemplateCoverPath(
                                                            userTemplate.user.uid,
                                                            item.template.name
                                                        ),
                                                        'template'
                                                    )
                                            "
                                        />
                                        <div class="template-main">
                                            <div class="template-name">
                                                {{ item.template.name }}
                                                <span
                                                    v-show="
                                                        checkTemplateHasUnsavedChanges(
                                                            userTemplate.user.uid,
                                                            item.template.name
                                                        )
                                                    "
                                                    class="unsaved-indicator"
                                                    title="有未保存的修改"
                                                ></span>
                                            </div>
                                            <div class="template-desc">
                                                {{ item.template.config.title || '无标题' }}
                                            </div>
                                        </div>
                                        <el-dropdown
                                            @command="
                                                (command: string) =>
                                                    handleTemplateCommand(
                                                        command,
                                                        userTemplate.user,
                                                        item.template,
                                                        item.folderId
                                                    )
                                            "
                                            @click.stop
                                            @mousedown.stop
                                            trigger="click"
                                            :disabled="templateLoading"
                                        >
                                            <el-button
                                                link
                                                size="small"
                                                class="template-menu-btn"
                                                @click.stop
                                                @mousedown.stop
                                            >
                                                <el-icon><more-filled /></el-icon>
                                            </el-button>
                                            <template #dropdown>
                                                <el-dropdown-menu>
                                                    <el-dropdown-item command="duplicate"
                                                        >复制</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="rename"
                                                        >重命名</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="move"
                                                        >移动到模板夹</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="move_root"
                                                        >移出模板夹</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="set_cover"
                                                        >设置封面</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="clear_cover"
                                                        >清除封面</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="move_up"
                                                        >上移</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="move_down"
                                                        >下移</el-dropdown-item
                                                    >
                                                    <el-dropdown-item command="delete" divided
                                                        >删除</el-dropdown-item
                                                    >
                                                </el-dropdown-menu>
                                            </template>
                                        </el-dropdown>
                                    </div>
                                </div>

                                <div
                                    v-if="
                                        !getTemplateRenderItems(
                                            userTemplate.user.uid,
                                            userTemplate.templates
                                        ).length
                                    "
                                    class="empty-folder"
                                >
                                    暂无模板
                                </div>

                                <div v-if="userTemplate.user.expired" class="expired-mask">
                                    <div class="expired-mask-content">
                                        <div class="expired-mask-title">账号登录状态已失效</div>
                                        <div class="expired-mask-desc">
                                            请重新登录后再编辑或选择模板
                                        </div>
                                        <el-button
                                            type="primary"
                                            size="small"
                                            @click.stop="showLoginDialog = true"
                                        >
                                            重新登录
                                        </el-button>
                                    </div>
                                </div>
                            </div>
                        </div>

                        <!-- 空状态 -->
                        <div v-if="userTemplates.length === 0" class="empty-users">
                            <el-empty description="暂无登录用户">
                                <el-button type="primary" @click="showLoginDialog = true">
                                    去登录
                                </el-button>
                            </el-empty>
                        </div>
                    </div>
                </div>
            </el-aside>
            <div
                class="sidebar-resizer"
                role="separator"
                aria-orientation="vertical"
                title="拖动调整模板栏宽度"
                @pointerdown="startSidebarResize"
            ></div>

            <!-- 主要内容区域 -->
            <el-main class="main-content">
                <div class="content-wrapper" ref="contentWrapperRef">
                    <div v-if="!selectedUser" class="no-selection">
                        <el-empty description="请选择用户和模板开始使用" />
                    </div>

                    <div v-else-if="!currentTemplateName" class="no-template">
                        <el-empty description="请选择模板或创建新模板">
                            <el-button type="primary" @click="showNewTemplateDialog = true">
                                新建模板
                            </el-button>
                        </el-empty>
                    </div>

                    <div v-else-if="currentForm" class="upload-form-container">
                        <el-tabs
                            v-model="activeSessionId"
                            type="card"
                            class="editor-tabs"
                            closable
                            @tab-remove="closeSessionTab"
                            @tab-click="handleSessionTabClick"
                        >
                            <el-tab-pane
                                v-for="session in editSessions"
                                :key="session.id"
                                :name="session.id"
                            >
                                <template #label>
                                    <span
                                        class="editor-tab-label"
                                        :class="{ 'auto-submit-tab': isSessionAutoSubmitting(session.id) }"
                                        @mousedown.middle.prevent
                                        @mouseup="event => handleTabLabelMouseup(event, session.id)"
                                    >
                                        {{ session.tabTitle }}
                                    </span>
                                </template>
                            </el-tab-pane>
                        </el-tabs>
                        <div class="form-header">
                            <div class="template-name-container">
                                <h3 class="edit-bv-template-disaplay" v-if="currentTemplate?.aid">
                                    编辑稿件：
                                </h3>
                                <el-tooltip
                                    v-if="currentTemplate?.aid"
                                    content="刷新稿件数据"
                                    placement="top"
                                >
                                    <el-icon
                                        class="refresh-btn"
                                        @click.stop="
                                            reloadTemplateFromAV(
                                                selectedUser.uid,
                                                currentTemplate.aid
                                            )
                                        "
                                    >
                                        <refresh />
                                    </el-icon>
                                </el-tooltip>
                                <h3
                                    v-if="!isEditingTemplateName"
                                    @click="handleTemplateNameEdit"
                                    class="template-name-display"
                                    :class="{ disabled: templateLoading }"
                                    :title="
                                        templateLoading
                                            ? '模板加载中，无法编辑'
                                            : '点击编辑模板名称'
                                    "
                                >
                                    {{ currentTemplateName }}
                                    <el-icon class="edit-hint-icon"><edit /></el-icon>
                                </h3>
                                <el-input
                                    v-else
                                    ref="templateNameInputRef"
                                    v-model="editingTemplateName"
                                    @blur="saveTemplateName"
                                    @keyup.enter="saveTemplateName"
                                    @keyup.esc="cancelEditTemplateName"
                                    class="template-name-input"
                                    size="large"
                                    :disabled="templateLoading"
                                />
                            </div>
                            <div class="header-actions">
                                <el-button @click="resetTemplate" :disabled="templateLoading"
                                    >放弃更改</el-button
                                >
                                <el-button
                                    type="primary"
                                    @click="saveTemplate"
                                    :disabled="templateLoading"
                                    >保存</el-button
                                >
                                <el-button
                                    @click="
                                        handleTemplateCommand('delete', selectedUser, {
                                            name: currentTemplateName,
                                            config: currentTemplate
                                        })
                                    "
                                    @click.stop
                                    trigger="click"
                                    type="danger"
                                    :disabled="templateLoading"
                                    >删除</el-button
                                >
                            </div>
                        </div>

                        <el-form :model="currentForm" label-width="80px" class="upload-form">
                            <!-- 基本信息 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.basic }"
                            >
                                <template #header>
                                    <div class="card-header" @click="toggleCardCollapsed('basic')">
                                        <span>基本信息</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('basic')"
                                                title="清空基本信息"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.basic }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.basic" class="card-content">
                                        <el-form-item label="视频标题" required>
                                            <el-input
                                                v-model="currentForm.title"
                                                placeholder="请输入视频标题"
                                                maxlength="80"
                                                show-word-limit
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item label="封面">
                                            <div class="cover-uploader-row">
                                                <div
                                                    class="cover-uploader"
                                                    action="#"
                                                    @click="handleCoverSelection"
                                                    v-loading="coverLoading"
                                                    :class="{ disabled: templateLoading }"
                                                >
                                                    <img
                                                        v-if="coverDisplayUrl && !coverLoading"
                                                        :src="coverDisplayUrl"
                                                        class="cover-image"
                                                    />
                                                    <el-icon
                                                        v-else-if="!coverLoading"
                                                        class="cover-uploader-icon"
                                                    >
                                                        <plus />
                                                    </el-icon>
                                                </div>

                                                <el-button
                                                    v-if="coverDisplayUrl && !coverLoading"
                                                    class="cover-clear-btn-side"
                                                    type="danger"
                                                    size="small"
                                                    @click.stop="clearCurrentCover"
                                                    :disabled="templateLoading"
                                                    title="清除封面"
                                                >
                                                    <el-icon><Close /></el-icon>
                                                </el-button>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="视频分区">
                                            <el-popover
                                                v-model:visible="categoryPopoverVisible"
                                                placement="bottom-start"
                                                :width="600"
                                                trigger="click"
                                                popper-class="category-popover"
                                            >
                                                <template #reference>
                                                    <el-button
                                                        class="category-trigger"
                                                        :type="
                                                            currentForm.tid ? 'primary' : 'default'
                                                        "
                                                        :disabled="templateLoading"
                                                    >
                                                        <span class="category-text">
                                                            <span v-if="selectedSubCategory">
                                                                {{ selectedCategory?.name }} >
                                                                {{ selectedSubCategory?.name }}
                                                            </span>
                                                            <span v-else class="placeholder"
                                                                >请选择分区</span
                                                            >
                                                        </span>
                                                        <el-icon class="arrow-icon">
                                                            <arrow-down />
                                                        </el-icon>
                                                    </el-button>
                                                </template>

                                                <div class="category-selector-panel">
                                                    <!-- 左侧主分区列表 -->
                                                    <div class="category-list">
                                                        <div
                                                            v-for="category in typeList"
                                                            :key="category.id"
                                                            class="category-item"
                                                            :class="{
                                                                active:
                                                                    selectedCategory?.id ===
                                                                    category.id
                                                            }"
                                                            @click="onCategoryChange(category.id)"
                                                        >
                                                            <span class="category-name">{{
                                                                category.name
                                                            }}</span>
                                                            <el-icon class="arrow-right">
                                                                <arrow-down
                                                                    style="
                                                                        transform: rotate(-90deg);
                                                                    "
                                                                />
                                                            </el-icon>
                                                        </div>
                                                    </div>

                                                    <!-- 右侧子分区列表 -->
                                                    <div class="subcategory-list">
                                                        <div
                                                            v-if="
                                                                selectedCategory &&
                                                                selectedCategory.children
                                                            "
                                                        >
                                                            <div
                                                                v-for="subCategory in selectedCategory.children"
                                                                :key="subCategory.id"
                                                                class="subcategory-item"
                                                                :class="{
                                                                    active:
                                                                        selectedSubCategory?.id ===
                                                                        subCategory.id
                                                                }"
                                                                @click="
                                                                    onSubCategoryChange(
                                                                        subCategory.id
                                                                    )
                                                                "
                                                                :title="
                                                                    subCategory.intro_original ||
                                                                    subCategory.desc
                                                                "
                                                            >
                                                                <div class="subcategory-content">
                                                                    <div class="subcategory-name">
                                                                        {{ subCategory.name }}
                                                                    </div>
                                                                    <div class="subcategory-desc">
                                                                        {{
                                                                            subCategory.desc !== ''
                                                                                ? subCategory.desc
                                                                                : subCategory.intro_original
                                                                        }}
                                                                    </div>
                                                                </div>
                                                            </div>
                                                        </div>
                                                        <div v-else class="empty-subcategory">
                                                            <el-empty
                                                                description="请选择左侧主分区"
                                                                :image-size="60"
                                                            />
                                                        </div>
                                                    </div>
                                                </div>
                                            </el-popover>
                                        </el-form-item>

                                        <el-form-item label="版权声明">
                                            <el-radio-group
                                                v-model="currentForm.copyright"
                                                :disabled="templateLoading"
                                            >
                                                <el-radio :value="1">自制</el-radio>
                                                <el-radio :value="2">转载</el-radio>
                                            </el-radio-group>
                                        </el-form-item>

                                        <el-form-item
                                            label="转载来源"
                                            v-if="currentForm.copyright === 2"
                                        >
                                            <el-input
                                                v-model="currentForm.source"
                                                placeholder="请填写转载来源"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 视频文件 -->
                            <el-card
                                class="form-section"
                                :class="{
                                    'drag-target': isDragOver,
                                    collapsed: cardCollapsed.videos
                                }"
                            >
                                <template #header>
                                    <div class="card-header">
                                        <div
                                            style="
                                                display: flex;
                                                align-items: center;
                                                gap: 12px;
                                                flex: 1;
                                            "
                                            @click="toggleCardCollapsed('videos')"
                                        >
                                            <span style="cursor: pointer">视频文件</span>
                                            <el-button
                                                type="success"
                                                size="small"
                                                @click.stop="checkVideoStatus"
                                                v-if="
                                                    currentForm.videos &&
                                                    currentForm.videos.length > 0 &&
                                                    currentTemplate?.aid
                                                "
                                                :disabled="templateLoading"
                                            >
                                                视频转码状态
                                            </el-button>
                                        </div>
                                        <div class="header-actions">
                                            <span v-if="isDragOver" class="drag-hint"
                                                >拖拽文件到此处添加</span
                                            >
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.videos }"
                                                @click="toggleCardCollapsed('videos')"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.videos" class="card-content">
                                        <VideoList
                                            v-model:videos="videos"
                                            :is-drag-over="isDragOver"
                                            :uploading="uploading"
                                            :template-title="currentTemplateName"
                                            :disabled="templateLoading"
                                            @select-video="selectVideoWithTauri"
                                            @clear-all-videos="clearAllVideos"
                                            @remove-file="removeUploadedFile"
                                            @create-upload="createUpload"
                                            @add-videos-to-form="handleAddVideosToForm"
                                            @submit-template="handleSubmitTemplate"
                                        />
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 标签设置 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.tags }"
                            >
                                <template #header>
                                    <div class="card-header" @click="toggleCardCollapsed('tags')">
                                        <span>标签设置</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('tags')"
                                                title="清空标签设置"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.tags }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.tags" class="card-content">
                                        <el-form-item label="视频标签">
                                            <TagView
                                                ref="tagViewRef"
                                                v-model="tags"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item v-if="!currentForm.aid" label="参与活动">
                                            <TopicView
                                                v-model="currentForm.mission_id"
                                                v-model:topic-id="currentForm.topic_id"
                                                :user-uid="selectedUser?.uid"
                                                mode="selector"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 视频描述 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.description }"
                            >
                                <template #header>
                                    <div
                                        class="card-header"
                                        @click="toggleCardCollapsed('description')"
                                    >
                                        <span>视频描述</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('description')"
                                                title="清空视频描述"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.description }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.description" class="card-content">
                                        <el-form-item label="简介">
                                            <el-input
                                                v-model="currentForm.desc"
                                                type="textarea"
                                                :rows="6"
                                                placeholder="请输入视频简介"
                                                maxlength="2000"
                                                show-word-limit
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item label="粉丝动态">
                                            <el-input
                                                v-model="currentForm.dynamic"
                                                placeholder="发布时的动态内容"
                                                maxlength="233"
                                                show-word-limit
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 高级选项 -->
                            <el-card
                                class="form-section"
                                :class="{ collapsed: cardCollapsed.advanced }"
                            >
                                <template #header>
                                    <div
                                        class="card-header"
                                        @click="toggleCardCollapsed('advanced')"
                                    >
                                        <span>高级选项</span>
                                        <div class="header-actions">
                                            <el-button
                                                type="danger"
                                                text
                                                size="small"
                                                @click.stop="clearCardContent('advanced')"
                                                title="清空高级选项"
                                                :disabled="templateLoading"
                                            >
                                                <el-icon><delete /></el-icon>
                                            </el-button>
                                            <el-icon
                                                class="collapse-icon"
                                                :class="{ collapsed: cardCollapsed.advanced }"
                                            >
                                                <arrow-down />
                                            </el-icon>
                                        </div>
                                    </div>
                                </template>

                                <el-collapse-transition>
                                    <div v-show="!cardCollapsed.advanced" class="card-content">
                                        <el-form-item label="开启水印">
                                            <div div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.watermark"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    开启 (本功能只对本次上传的视频生效)
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>
                                        <el-form-item v-if="!currentForm.aid" label="定时发布">
                                            <el-date-picker
                                                v-model="dtimeDate"
                                                type="datetime"
                                                placeholder="选择发布时间"
                                                format="YYYY-MM-DD HH:mm:ss"
                                                :disabled="templateLoading"
                                                :disabled-date="
                                                    (date: Date) => {
                                                        const now = new Date()
                                                        const twoHoursLater = new Date(
                                                            now.getTime() + 2 * 60 * 60 * 1000
                                                        )
                                                        const fifteenDaysLater = new Date(
                                                            now.getTime() + 15 * 24 * 60 * 60 * 1000
                                                        )

                                                        return (
                                                            date < twoHoursLater ||
                                                            date > fifteenDaysLater
                                                        )
                                                    }
                                                "
                                            />
                                        </el-form-item>

                                        <el-form-item label="字幕设置">
                                            <el-checkbox
                                                v-model="currentForm.open_subtitle"
                                                :disabled="templateLoading"
                                            >
                                                开启字幕功能
                                            </el-checkbox>
                                        </el-form-item>

                                        <el-form-item label="互动功能">
                                            <div class="interactive-setting-row">
                                                <el-checkbox
                                                    v-model="currentForm.interactive"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    开启
                                                </el-checkbox>

                                                <el-tooltip
                                                    content="勾选后本视频将被投稿为互动视频，需在规定时间内完成剧情树配置，否则系统可能回收稿件。"
                                                    placement="top"
                                                >
                                                    <el-icon
                                                        class="interactive-help-icon"
                                                        @click.stop="showInteractiveInfoDialog"
                                                    >
                                                        <QuestionFilled />
                                                    </el-icon>
                                                </el-tooltip>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="加入合集">
                                            <SeasonView
                                                v-model="currentForm.season_id"
                                                v-model:section-id="currentForm.section_id"
                                                :user-uid="selectedUser?.uid"
                                                :disabled="templateLoading"
                                            />
                                        </el-form-item>

                                        <el-form-item label="音质设置">
                                            <div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.dolby"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    杜比音效
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.lossless_music"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    无损音乐
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="内容设置">
                                            <div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.no_reprint"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    禁止转载
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.open_elec"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    开启充电
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="互动管理">
                                            <div class="checkbox-group">
                                                <el-checkbox
                                                    v-model="currentForm.up_selection_reply"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    UP主精选评论
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.up_close_reply"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    关闭评论
                                                </el-checkbox>
                                                <el-checkbox
                                                    v-model="currentForm.up_close_danmu"
                                                    :true-value="1"
                                                    :false-value="0"
                                                    :disabled="templateLoading"
                                                >
                                                    关闭弹幕
                                                </el-checkbox>
                                            </div>
                                        </el-form-item>

                                        <el-form-item label="可见性">
                                            <el-checkbox
                                                v-model="currentForm.is_only_self"
                                                :true-value="1"
                                                :false-value="0"
                                                :disabled="templateLoading"
                                            >
                                                仅自己可见
                                            </el-checkbox>
                                        </el-form-item>
                                    </div>
                                </el-collapse-transition>
                            </el-card>

                            <!-- 上传操作区域 -->
                            <div class="upload-actions">
                                <el-button
                                    type="primary"
                                    size="large"
                                    :loading="submitting"
                                    @click="submitTemplate"
                                    :disabled="
                                        templateLoading ||
                                        !currentForm.videos ||
                                        currentForm.videos.length === 0 ||
                                        !currentForm.title ||
                                        currentForm.title.trim() === ''
                                    "
                                >
                                    <el-icon v-if="!allFilesUploaded && !submitting"
                                        ><loading
                                    /></el-icon>
                                    <el-icon v-else-if="!submitting"><check /></el-icon>
                                    {{
                                        !getCurrentAutoSubmitting
                                            ? currentTemplate?.aid
                                                ? '编辑稿件'
                                                : '新增稿件'
                                            : '上传完成后自动提交'
                                    }}
                                </el-button>
                                <div class="form-tip" v-if="lastSubmit">
                                    最后提交时间: {{ lastSubmit }}
                                </div>
                            </div>
                        </el-form>
                    </div>
                    <div v-else class="no-template">
                        <el-empty description="当前标签页草稿不可用，请重新打开模板" />
                    </div>
                </div>
            </el-main>
        </el-container>

        <!-- 新建模板组件 -->
        <NewTemplete
            ref="newTemplateRef"
            v-model="showNewTemplateDialog"
            @template-created="handleTemplateCreated"
        />

        <!-- 移动模板到模板夹 -->
        <el-dialog v-model="moveDialogVisible" title="移动到模板夹" width="420px">
            <div class="move-dialog-content">
                <el-select
                    v-model="moveDialogTargetFolderId"
                    placeholder="选择模板夹"
                    class="move-dialog-select"
                >
                    <el-option label="未分组" value="" />
                    <el-option
                        v-for="folder in getUserOrganizer(moveDialogUserUid || 0).folders"
                        :key="folder.id"
                        :label="folder.name"
                        :value="folder.id"
                    />
                </el-select>
                <el-button size="small" @click="createFolderFromMoveDialog">
                    新建模板夹
                </el-button>
            </div>
            <template #footer>
                <div class="dialog-footer">
                    <el-button @click="moveDialogVisible = false">取消</el-button>
                    <el-button type="primary" @click="confirmMoveDialog">确定</el-button>
                </div>
            </template>
        </el-dialog>

        <!-- 登录对话框 -->
        <el-dialog
            v-model="showLoginDialog"
            width="500px"
            :show-close="false"
            :close-on-click-modal="true"
            :close-on-press-escape="false"
            :before-close="handleLoginDialogClose"
            class="login-dialog"
            top="5vh"
        >
            <div class="login-dialog-content" @click.stop>
                <LoginView
                    @login-success="handleLoginSuccess"
                    @loading-change="loginLoading = $event"
                />
            </div>
        </el-dialog>

        <!-- 用户配置对话框 -->
        <UserConfig v-model="userConfigVisible" :user="configUser" />

        <!-- 视频状态对话框 -->
        <VideoStatus
            v-model="showVideoStatusDialog"
            :videos="currentForm?.videos || []"
            :user="selectedUser"
            :template-aid="currentTemplate?.aid"
            @reload-template="
                () =>
                    selectedUser &&
                    currentTemplate?.aid &&
                    reloadTemplateFromAV(selectedUser.uid, currentTemplate.aid)
            "
        />

        <!-- 全局配置对话框 -->
        <GlobalConfigView v-model="showGlobalConfigDialog" />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick, watch, onUnmounted } from 'vue'
import { v4 as uuidv4 } from 'uuid'
import { useAuthStore } from '../stores/auth'
import { useUserConfigStore, TemplateConfig } from '../stores/user_config'
import { useUtilsStore } from '../stores/utils'
import { useUploadStore } from '../stores/upload'
import { ElMessageBox } from 'element-plus'
import {
    ArrowDown,
    Plus,
    MoreFilled,
    UploadFilled,
    User,
    Check,
    Edit,
    Setting,
    Refresh,
    Delete,
    Close,
    QuestionFilled
} from '@element-plus/icons-vue'
import { open, save } from '@tauri-apps/plugin-dialog'
import { convertFileSrc, invoke } from '@tauri-apps/api/core'
import { readFile, stat } from '@tauri-apps/plugin-fs'
import { openUrl } from '@tauri-apps/plugin-opener'
import { listen } from '@tauri-apps/api/event'
import LoginView from '../components/LoginView.vue'
import UserConfig from '../components/UserConfig.vue'
import TopicView from '../components/TopicView.vue'
import SeasonView from '../components/SeasonView.vue'
import UploadQueue from '../components/UploadQueue.vue'
import GlobalConfigView from '../components/GlobalConfig.vue'
import NewTemplete from '../components/NewTemplete.vue'
import VideoList from '../components/VideoList.vue'
import UserList from '../components/UserList.vue'
import VideoStatus from '../components/VideoStatus.vue'
import TagView from '../components/TagView.vue'

const authStore = useAuthStore()
const userConfigStore = useUserConfigStore()
const uploadStore = useUploadStore()
const utilsStore = useUtilsStore()

type SortMode = 'manual' | 'az'

interface TemplateFolder {
    id: string
    name: string
    order: string[]
    collapsed?: boolean
    coverPath?: string
}

interface UserTemplateOrganizer {
    sortMode: SortMode
    ungroupedOrder: string[]
    folders: TemplateFolder[]
    templateCovers?: Record<string, string>
}

interface TemplateRenderItemFolder {
    kind: 'folder'
    id: string
    folderId: string
    name: string
    count: number
}

interface TemplateRenderItemTemplate {
    kind: 'template'
    id: string
    folderId?: string
    template: { name: string; config: TemplateConfig }
}

type TemplateRenderItem = TemplateRenderItemFolder | TemplateRenderItemTemplate

interface TemplateEditSession {
    id: string
    uid: number
    username: string
    templateName: string
    tabTitle: string
    createdAt: number
    updatedAt: number
    draft: TemplateConfig
}

// 计算属性
const loginUsers = computed(() => authStore.loginUsers)
const userTemplates = computed(() => userConfigStore.userTemplates)
const typeList = computed(() => utilsStore.typelist)

const currentVer = ref<string>('')

// 封面显示URL
const coverDisplayUrl = ref<string>('')
const coverLoading = ref<boolean>(false)

// 响应式数据
const selectedUser = ref<any>(null)
const currentTemplateName = ref<string>('')
const editSessions = ref<TemplateEditSession[]>([])
const activeSessionId = ref<string>('')
const showNewTemplateDialog = ref(false)
const showLoginDialog = ref(false)
const showGlobalConfigDialog = ref(false)
const loginLoading = ref(false)
const uploading = ref(false)
const submitting = ref(false)
const templateLoading = ref(false) // 模板加载状态锁

// 视频状态对话框
const showVideoStatusDialog = ref(false)
const interactiveConfirmShown = ref<Record<string, boolean>>({})
const interactiveDialogOpening = ref(false)

// 组件引用
const newTemplateRef = ref<InstanceType<typeof NewTemplete> | null>(null)
const tagViewRef = ref<InstanceType<typeof TagView> | null>(null)
// 自动提交状态记录 - 记录每个模板的自动提交状态
const autoSubmittingRecord = ref<Record<string, boolean>>({})
// 全局自动提交检查间隔
let autoSubmitInterval: number | null = null

// 高亮显示自动提交状态的开关
const highlightAutoSubmitting = ref<boolean>(
    localStorage.getItem('highlightAutoSubmitting') === 'true'
)

// 监听高亮开关变化，保存到localStorage
watch(highlightAutoSubmitting, newValue => {
    localStorage.setItem('highlightAutoSubmitting', String(newValue))
})

watch(
    () => showGlobalConfigDialog.value,
    newValue => {
        if (!newValue) {
            reloadLocalUiSettings()
        }
    }
)

const TEMPLATE_ORGANIZER_KEY = 'template-organizer-v1'
const TEMPLATE_COVER_DEFAULT_KEY = 'template-cover-default'
const FOLDER_COVER_DEFAULT_KEY = 'folder-cover-default'
const TEMPLATE_SESSIONS_KEY = 'template-edit-sessions-v1'
const TEMPLATE_TAB_MAX_KEY = 'template-tab-max'
const TEMPLATE_COVER_SIZE_KEY = 'template-cover-size'
const SIDEBAR_WIDTH_KEY = 'template-sidebar-width'
const SIDEBAR_MIN_WIDTH = 280
const DEFAULT_TEMPLATE_COVER = '/noface.jpg'
const DEFAULT_FOLDER_COVER = '/tauri.svg'
const templateOrganizer = ref<Record<number, UserTemplateOrganizer>>({})
const coverVersionMap = ref<Record<string, number>>({})
const sidebarRef = ref<any>(null)
const sidebarWidth = ref<number>(320)
let sidebarResizeCleanup: (() => void) | null = null
const moveDialogVisible = ref(false)
const moveDialogUserUid = ref<number | null>(null)
const moveDialogTemplateName = ref<string>('')
const moveDialogTemplateFolderId = ref<string | null>(null)
const moveDialogTargetFolderId = ref<string>('')

let sessionPersistTimer: number | null = null
const templateCoverSizeValue = ref(48)
const draggingTemplate = ref<{
    uid: number
    templateName: string
    folderId?: string
} | null>(null)

const getTemplateTabMax = () => {
    const raw = Number.parseInt(localStorage.getItem(TEMPLATE_TAB_MAX_KEY) || '', 10)
    if (!Number.isFinite(raw)) return 15
    return Math.min(30, Math.max(3, raw))
}

const getTemplateCoverSize = () => {
    const raw = Number.parseInt(localStorage.getItem(TEMPLATE_COVER_SIZE_KEY) || '', 10)
    if (!Number.isFinite(raw)) return 48
    return Math.min(96, Math.max(40, raw))
}

const reloadLocalUiSettings = () => {
    templateCoverSizeValue.value = getTemplateCoverSize()
}

const templateCoverStyle = computed(() => {
    const size = templateCoverSizeValue.value
    return {
        width: `${size}px`,
        height: `${size}px`
    }
})

const cloneTemplateConfig = (template?: TemplateConfig | null): TemplateConfig => {
    return JSON.parse(JSON.stringify(template || userConfigStore.createDefaultTemplate()))
}

const touchActiveSession = () => {
    const session = editSessions.value.find(item => item.id === activeSessionId.value)
    if (!session) return
    session.updatedAt = Date.now()
}

const persistSessionsNow = () => {
    try {
        const payload = {
            activeSessionId: activeSessionId.value,
            sessions: editSessions.value
        }
        localStorage.setItem(TEMPLATE_SESSIONS_KEY, JSON.stringify(payload))
    } catch (error) {
        console.error('保存编辑会话失败:', error)
    }
}

const schedulePersistSessions = () => {
    if (sessionPersistTimer) {
        clearTimeout(sessionPersistTimer)
    }
    sessionPersistTimer = window.setTimeout(() => {
        persistSessionsNow()
    }, 200)
}

const getDefaultOrganizer = (): UserTemplateOrganizer => ({
    sortMode: 'manual',
    ungroupedOrder: [],
    folders: [],
    templateCovers: {}
})

const saveTemplateOrganizer = () => {
    localStorage.setItem(TEMPLATE_ORGANIZER_KEY, JSON.stringify(templateOrganizer.value))
}

const loadTemplateOrganizer = () => {
    try {
        const raw = localStorage.getItem(TEMPLATE_ORGANIZER_KEY)
        if (!raw) return
        const parsed = JSON.parse(raw)
        if (parsed && typeof parsed === 'object') {
            templateOrganizer.value = parsed
        }
    } catch (error) {
        console.error('读取模板排序/模板夹配置失败:', error)
    }
}

const clampSidebarWidth = (width: number) => {
    const maxWidth = Math.max(SIDEBAR_MIN_WIDTH, Math.floor(window.innerWidth * 0.6))
    return Math.min(maxWidth, Math.max(SIDEBAR_MIN_WIDTH, Math.floor(width)))
}

const loadSidebarWidth = () => {
    const raw = localStorage.getItem(SIDEBAR_WIDTH_KEY)
    if (!raw) {
        sidebarWidth.value = clampSidebarWidth(320)
        return
    }
    const parsed = Number.parseInt(raw, 10)
    sidebarWidth.value = Number.isFinite(parsed) ? clampSidebarWidth(parsed) : clampSidebarWidth(320)
}

const saveSidebarWidth = () => {
    localStorage.setItem(SIDEBAR_WIDTH_KEY, String(sidebarWidth.value))
}

const sidebarInlineStyle = computed(() => ({
    width: `${sidebarWidth.value}px`,
    flex: `0 0 ${sidebarWidth.value}px`
}))

const startSidebarResize = (event: PointerEvent) => {
    const sidebarElement = sidebarRef.value?.$el || sidebarRef.value
    if (!sidebarElement || typeof sidebarElement.getBoundingClientRect !== 'function') return
    event.preventDefault()
    ;(event.currentTarget as HTMLElement | null)?.setPointerCapture?.(event.pointerId)
    const startX = event.clientX
    const startWidth = sidebarElement.getBoundingClientRect().width

    const onMove = (moveEvent: PointerEvent) => {
        const delta = moveEvent.clientX - startX
        sidebarWidth.value = clampSidebarWidth(startWidth + delta)
    }

    const onUp = async (upEvent: PointerEvent) => {
        ;(event.currentTarget as HTMLElement | null)?.releasePointerCapture?.(upEvent.pointerId)
        document.body.classList.remove('sidebar-resizing')
        saveSidebarWidth()
        await utilsStore.log('log', `模板栏宽度已调整: ${sidebarWidth.value}px`)
        window.removeEventListener('pointermove', onMove)
        window.removeEventListener('pointerup', onUp)
    }

    document.body.classList.add('sidebar-resizing')
    window.addEventListener('pointermove', onMove)
    window.addEventListener('pointerup', onUp)
}

const getUserOrganizer = (uid: number): UserTemplateOrganizer => {
    if (!templateOrganizer.value[uid]) {
        templateOrganizer.value[uid] = getDefaultOrganizer()
    }
    return templateOrganizer.value[uid]
}

const ensureTemplateInOrganizer = (uid: number, templateName: string) => {
    const organizer = getUserOrganizer(uid)
    const existsInFolder = organizer.folders.some(folder => folder.order.includes(templateName))
    const existsInRoot = organizer.ungroupedOrder.includes(templateName)
    if (!existsInFolder && !existsInRoot) {
        organizer.ungroupedOrder.push(templateName)
    }
}

const removeTemplateFromOrganizer = (uid: number, templateName: string) => {
    const organizer = getUserOrganizer(uid)
    organizer.ungroupedOrder = organizer.ungroupedOrder.filter(name => name !== templateName)
    organizer.folders.forEach(folder => {
        folder.order = folder.order.filter(name => name !== templateName)
    })
}

const sanitizeUserOrganizer = (
    uid: number,
    templates: Array<{ name: string; config: TemplateConfig }>
) => {
    const organizer = getUserOrganizer(uid)
    const existingNames = new Set(templates.map(t => t.name))
    const seen = new Set<string>()

    organizer.folders = organizer.folders.map(folder => {
        const uniqueOrder: string[] = []
        for (const name of folder.order) {
            if (!existingNames.has(name) || seen.has(name)) continue
            if (!uniqueOrder.includes(name)) {
                uniqueOrder.push(name)
                seen.add(name)
            }
        }
        return {
            ...folder,
            collapsed: folder.collapsed ?? true,
            coverPath: folder.coverPath || '',
            order: uniqueOrder
        }
    })

    organizer.ungroupedOrder = organizer.ungroupedOrder.filter(name => {
        if (!existingNames.has(name) || seen.has(name)) return false
        seen.add(name)
        return true
    })

    for (const template of templates) {
        if (!seen.has(template.name)) {
            organizer.ungroupedOrder.push(template.name)
            seen.add(template.name)
        }
    }

    organizer.templateCovers = organizer.templateCovers || {}
    templateOrganizer.value[uid] = organizer
}

const syncTemplateOrganizer = () => {
    for (const userTemplate of userTemplates.value) {
        sanitizeUserOrganizer(userTemplate.user.uid, userTemplate.templates)
    }
    saveTemplateOrganizer()
}

const getTemplateSortMode = (uid: number): SortMode => getUserOrganizer(uid).sortMode

const updateTemplateSortMode = (uid: number, mode: string) => {
    const organizer = getUserOrganizer(uid)
    organizer.sortMode = mode === 'az' ? 'az' : 'manual'
    saveTemplateOrganizer()
}

const selectTemplateCover = async (uid: number, templateName: string) => {
    try {
        const selected = await open({
            multiple: false,
            defaultPath: getDefaultCoverDir() || undefined,
            filters: [
                {
                    name: 'Image',
                    extensions: ['jpg', 'jpeg', 'png', 'pjp', 'pjpeg', 'jiff', 'gif']
                }
            ]
        })
        if (!selected || selected.length === 0) {
            utilsStore.showMessage('未选择任何封面文件', 'warning')
            return
        }
        const path = Array.isArray(selected) ? selected[0] : selected
        const storedPath = await invoke<string>('store_local_cover', {
            uid,
            key: templateName,
            sourcePath: path,
            isFolder: false
        })
        await stat(storedPath)
        setTemplateCoverPath(uid, templateName, storedPath)
        const folder = path.split(/[/\\]/).slice(0, -1).join('\\')
        if (folder) setDefaultCoverDir(folder)
        await utilsStore.log(
            'log',
            `模板封面已更新: uid=${uid}, template=${templateName}, path=${storedPath}`
        )
        utilsStore.showMessage('模板封面已更新', 'success')
    } catch (error) {
        console.error('模板封面选择失败:', error)
        utilsStore.showMessage(`模板封面选择失败: ${error}`, 'error')
    }
}

const selectFolderCover = async (uid: number, folderId: string) => {
    try {
        const selected = await open({
            multiple: false,
            defaultPath: getDefaultCoverDir() || undefined,
            filters: [
                {
                    name: 'Image',
                    extensions: ['jpg', 'jpeg', 'png', 'pjp', 'pjpeg', 'jiff', 'gif']
                }
            ]
        })
        if (!selected || selected.length === 0) {
            utilsStore.showMessage('未选择任何封面文件', 'warning')
            return
        }
        const path = Array.isArray(selected) ? selected[0] : selected
        const storedPath = await invoke<string>('store_local_cover', {
            uid,
            key: folderId,
            sourcePath: path,
            isFolder: true
        })
        await stat(storedPath)
        setFolderCoverPath(uid, folderId, storedPath)
        const folder = path.split(/[/\\]/).slice(0, -1).join('\\')
        if (folder) setDefaultCoverDir(folder)
        await utilsStore.log(
            'log',
            `模板夹封面已更新: uid=${uid}, folder=${folderId}, path=${storedPath}`
        )
        utilsStore.showMessage('模板夹封面已更新', 'success')
    } catch (error) {
        console.error('模板夹封面选择失败:', error)
        utilsStore.showMessage(`模板夹封面选择失败: ${error}`, 'error')
    }
}

const DEFAULT_COVER_DIR_KEY = 'default-cover-dir'
const DEFAULT_VIDEO_DIR_KEY = 'default-video-dir'

const getDefaultCoverDir = () => localStorage.getItem(DEFAULT_COVER_DIR_KEY) || ''
const setDefaultCoverDir = (path: string) =>
    localStorage.setItem(DEFAULT_COVER_DIR_KEY, path)
const getDefaultVideoDir = () => localStorage.getItem(DEFAULT_VIDEO_DIR_KEY) || ''
const setDefaultVideoDir = (path: string) =>
    localStorage.setItem(DEFAULT_VIDEO_DIR_KEY, path)

const sortTemplates = (
    templates: Array<{ name: string; config: TemplateConfig }>,
    mode: SortMode
) => {
    if (mode === 'az') {
        return [...templates].sort((a, b) => a.name.localeCompare(b.name, 'zh-CN'))
    }
    return templates
}

const getDefaultTemplateCover = () =>
    localStorage.getItem(TEMPLATE_COVER_DEFAULT_KEY) || DEFAULT_TEMPLATE_COVER

const getDefaultFolderCover = () =>
    localStorage.getItem(FOLDER_COVER_DEFAULT_KEY) || DEFAULT_FOLDER_COVER

const normalizeCoverPath = (coverPath?: string) => {
    if (!coverPath) return ''
    if (coverPath.startsWith('file:///')) {
        return decodeURIComponent(coverPath.replace('file:///', ''))
    }
    if (coverPath.startsWith('file://')) {
        return decodeURIComponent(coverPath.replace('file://', ''))
    }
    return coverPath
}

const guessMimeType = (filePath: string) => {
    const lower = filePath.toLowerCase()
    if (lower.endsWith('.png')) return 'image/png'
    if (lower.endsWith('.webp')) return 'image/webp'
    if (lower.endsWith('.gif')) return 'image/gif'
    return 'image/jpeg'
}

const toDataUrlFromPath = async (filePath: string) => {
    const bytes = await readFile(filePath)
    let binary = ''
    const chunkSize = 0x8000
    for (let i = 0; i < bytes.length; i += chunkSize) {
        const chunk = bytes.slice(i, i + chunkSize)
        binary += String.fromCharCode(...chunk)
    }
    const base64 = btoa(binary)
    return `data:${guessMimeType(filePath)};base64,${base64}`
}

const bumpCoverVersion = (coverPath?: string) => {
    const normalized = normalizeCoverPath(coverPath)
    if (!normalized) return
    coverVersionMap.value[normalized] = Date.now()
}

const resolveCoverSrc = (coverPath: string | undefined, fallbackType: 'template' | 'folder') => {
    const normalized = normalizeCoverPath(coverPath)
    if (!normalized) {
        return fallbackType === 'template' ? getDefaultTemplateCover() : getDefaultFolderCover()
    }
    if (
        normalized.startsWith('http://') ||
        normalized.startsWith('https://') ||
        normalized.startsWith('data:')
    ) {
        return normalized
    }
    const normalizedForTauri = normalized.replace(/\\/g, '/')
    const src = convertFileSrc(normalizedForTauri)
    const version = coverVersionMap.value[normalized]
    return version ? `${src}?v=${version}` : src
}

const getTemplateCoverPath = (uid: number, templateName: string) => {
    const organizer = getUserOrganizer(uid)
    return organizer.templateCovers?.[templateName] || ''
}

const setTemplateCoverPath = (uid: number, templateName: string, coverPath: string) => {
    const organizer = getUserOrganizer(uid)
    organizer.templateCovers = organizer.templateCovers || {}
    organizer.templateCovers[templateName] = coverPath
    bumpCoverVersion(coverPath)
    saveTemplateOrganizer()
}

const clearTemplateCoverPath = (uid: number, templateName: string) => {
    const organizer = getUserOrganizer(uid)
    if (organizer.templateCovers && organizer.templateCovers[templateName]) {
        const oldPath = organizer.templateCovers[templateName]
        delete organizer.templateCovers[templateName]
        bumpCoverVersion(oldPath)
        saveTemplateOrganizer()
    }
}

const getFolderCoverPath = (uid: number, folderId: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    return folder?.coverPath || ''
}

const setFolderCoverPath = (uid: number, folderId: string, coverPath: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    if (!folder) return
    folder.coverPath = coverPath
    bumpCoverVersion(coverPath)
    saveTemplateOrganizer()
}

const clearFolderCoverPath = (uid: number, folderId: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    if (!folder) return
    const oldPath = folder.coverPath
    folder.coverPath = ''
    bumpCoverVersion(oldPath)
    saveTemplateOrganizer()
}

const handleCoverError = async (
    event: Event,
    coverPath: string | undefined,
    fallbackType: 'template' | 'folder'
) => {
    const target = event.target as HTMLImageElement
    const normalized = normalizeCoverPath(coverPath)
    if (normalized) {
        try {
            const dataUrl = await toDataUrlFromPath(normalized)
            target.src = dataUrl
            return
        } catch (error) {
            await utilsStore.log(
                'error',
                `封面读取失败，回退默认封面: path=${normalized}, error=${JSON.stringify(error)}`
            )
        }
    }
    target.src = fallbackType === 'template' ? getDefaultTemplateCover() : getDefaultFolderCover()
}

const toggleFolderCollapsed = (uid: number, folderId: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    if (!folder) return
    folder.collapsed = !folder.collapsed
    saveTemplateOrganizer()
}

const isFolderCollapsed = (uid: number, folderId: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    return folder?.collapsed ?? true
}

const getTemplateRenderItems = (
    uid: number,
    templates: Array<{ name: string; config: TemplateConfig }>
): TemplateRenderItem[] => {
    const organizer = getUserOrganizer(uid)
    const byName = new Map(templates.map(template => [template.name, template]))
    const mode = organizer.sortMode
    const items: TemplateRenderItem[] = []

    organizer.folders.forEach(folder => {
        const folderTemplates = folder.order
            .map(name => byName.get(name))
            .filter((template): template is { name: string; config: TemplateConfig } => !!template)
        const finalFolderTemplates = sortTemplates(folderTemplates, mode)
        if (finalFolderTemplates.length === 0) return

        items.push({
            kind: 'folder',
            id: `folder-${folder.id}`,
            folderId: folder.id,
            name: folder.name,
            count: finalFolderTemplates.length
        })

        if (!folder.collapsed) {
            finalFolderTemplates.forEach(template => {
                items.push({
                    kind: 'template',
                    id: `template-${folder.id}-${template.name}`,
                    folderId: folder.id,
                    template
                })
            })
        }
    })

    const ungroupedTemplates = organizer.ungroupedOrder
        .map(name => byName.get(name))
        .filter((template): template is { name: string; config: TemplateConfig } => !!template)
    const finalUngroupedTemplates = sortTemplates(ungroupedTemplates, mode)

    finalUngroupedTemplates.forEach(template => {
        items.push({
            kind: 'template',
            id: `template-root-${template.name}`,
            template
        })
    })

    return items
}

const createTemplateFolder = async (uid: number): Promise<string | null> => {
    try {
        const { value } = await ElMessageBox.prompt('请输入模板夹名称', '新建模板夹', {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            inputPlaceholder: '例如：翻唱、游戏实况、投稿备份'
        })
        const folderName = value.trim()
        if (!folderName) {
            utilsStore.showMessage('模板夹名称不能为空', 'error')
            return null
        }
        const organizer = getUserOrganizer(uid)
        if (organizer.folders.some(folder => folder.name === folderName)) {
            utilsStore.showMessage('模板夹名称已存在', 'error')
            return null
        }
        const folderId = uuidv4()
        organizer.folders.push({
            id: folderId,
            name: folderName,
            order: [],
            collapsed: true,
            coverPath: ''
        })
        saveTemplateOrganizer()
        utilsStore.showMessage('模板夹创建成功', 'success')
        return folderId
    } catch (error) {
        if (error !== 'cancel') {
            console.error('新建模板夹失败:', error)
            utilsStore.showMessage(`新建模板夹失败: ${error}`, 'error')
        }
    }
    return null
}

const renameTemplateFolder = async (uid: number, folderId: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    if (!folder) return

    try {
        const { value } = await ElMessageBox.prompt('请输入新的模板夹名称', '重命名模板夹', {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            inputPlaceholder: '请输入模板夹名称',
            inputValue: folder.name
        })
        const newName = value.trim()
        if (!newName) {
            utilsStore.showMessage('模板夹名称不能为空', 'error')
            return
        }
        if (organizer.folders.some(item => item.name === newName && item.id !== folderId)) {
            utilsStore.showMessage('模板夹名称已存在', 'error')
            return
        }
        folder.name = newName
        saveTemplateOrganizer()
        utilsStore.showMessage('模板夹重命名成功', 'success')
    } catch (error) {
        if (error !== 'cancel') {
            console.error('重命名模板夹失败:', error)
            utilsStore.showMessage(`重命名模板夹失败: ${error}`, 'error')
        }
    }
}

const deleteTemplateFolder = async (uid: number, folderId: string) => {
    const organizer = getUserOrganizer(uid)
    const folder = organizer.folders.find(item => item.id === folderId)
    if (!folder) return

    try {
        await ElMessageBox.confirm(
            `删除模板夹 "${folder.name}" 后，模板会移回未分组列表，是否继续？`,
            '删除模板夹',
            {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }
        )

        organizer.ungroupedOrder.push(...folder.order.filter(name => !organizer.ungroupedOrder.includes(name)))
        organizer.folders = organizer.folders.filter(item => item.id !== folderId)
        saveTemplateOrganizer()
        utilsStore.showMessage('模板夹已删除', 'success')
    } catch (error) {
        if (error !== 'cancel') {
            console.error('删除模板夹失败:', error)
            utilsStore.showMessage(`删除模板夹失败: ${error}`, 'error')
        }
    }
}

const openMoveDialog = (uid: number, templateName: string, folderId?: string) => {
    moveDialogUserUid.value = uid
    moveDialogTemplateName.value = templateName
    moveDialogTemplateFolderId.value = folderId || null
    moveDialogTargetFolderId.value = folderId || ''
    moveDialogVisible.value = true
}

const confirmMoveDialog = async () => {
    if (!moveDialogUserUid.value || !moveDialogTemplateName.value) {
        moveDialogVisible.value = false
        return
    }

    const uid = moveDialogUserUid.value
    const templateName = moveDialogTemplateName.value
    const organizer = getUserOrganizer(uid)

    removeTemplateFromOrganizer(uid, templateName)

    if (!moveDialogTargetFolderId.value) {
        organizer.ungroupedOrder.push(templateName)
    } else {
        const targetFolder = organizer.folders.find(
            folder => folder.id === moveDialogTargetFolderId.value
        )
        if (targetFolder) {
            targetFolder.order.push(templateName)
        } else {
            organizer.ungroupedOrder.push(templateName)
        }
    }
    saveTemplateOrganizer()
    utilsStore.showMessage('模板移动成功', 'success')
    moveDialogVisible.value = false
}

const createFolderFromMoveDialog = async () => {
    if (!moveDialogUserUid.value) return
    const newFolderId = await createTemplateFolder(moveDialogUserUid.value)
    if (newFolderId) {
        moveDialogTargetFolderId.value = newFolderId
    }
}

const moveTemplateToRoot = (uid: number, templateName: string) => {
    const organizer = getUserOrganizer(uid)
    removeTemplateFromOrganizer(uid, templateName)
    organizer.ungroupedOrder.push(templateName)
    saveTemplateOrganizer()
    utilsStore.showMessage('模板已移出模板夹', 'success')
}

const moveTemplatePosition = (uid: number, templateName: string, direction: 'up' | 'down') => {
    const organizer = getUserOrganizer(uid)
    organizer.sortMode = 'manual'

    const folder = organizer.folders.find(item => item.order.includes(templateName))
    const order = folder ? folder.order : organizer.ungroupedOrder
    const index = order.indexOf(templateName)
    if (index < 0) return
    const target = direction === 'up' ? index - 1 : index + 1
    if (target < 0 || target >= order.length) return

    ;[order[index], order[target]] = [order[target], order[index]]
    saveTemplateOrganizer()
}

const isTemplateDragEnabled = (uid: number, expired: boolean) => {
    if (expired || templateLoading.value) return false
    return getTemplateSortMode(uid) === 'manual'
}

const getTemplateOrderRef = (organizer: UserTemplateOrganizer, folderId?: string) => {
    if (!folderId) return organizer.ungroupedOrder
    const folder = organizer.folders.find(item => item.id === folderId)
    return folder ? folder.order : organizer.ungroupedOrder
}

const onTemplateDragStart = (
    event: DragEvent,
    uid: number,
    templateName: string,
    folderId?: string
) => {
    if (!isTemplateDragEnabled(uid, false)) return
    draggingTemplate.value = { uid, templateName, folderId }
    event.dataTransfer?.setData('text/plain', templateName)
    if (event.dataTransfer) {
        event.dataTransfer.effectAllowed = 'move'
    }
}

const onTemplateDragOver = (
    event: DragEvent,
    uid: number,
    _templateName: string,
    _folderId?: string
) => {
    if (!draggingTemplate.value) return
    if (draggingTemplate.value.uid !== uid) return
    if (!isTemplateDragEnabled(uid, false)) return
    if (event.dataTransfer) {
        event.dataTransfer.dropEffect = 'move'
    }
}

const onTemplateDrop = (
    _event: DragEvent,
    uid: number,
    targetTemplateName: string,
    targetFolderId?: string
) => {
    const dragging = draggingTemplate.value
    draggingTemplate.value = null
    if (!dragging) return
    if (dragging.uid !== uid) return
    if (dragging.templateName === targetTemplateName && dragging.folderId === targetFolderId) return
    if (!isTemplateDragEnabled(uid, false)) return

    const organizer = getUserOrganizer(uid)
    const sourceOrder = getTemplateOrderRef(organizer, dragging.folderId)
    const targetOrder = getTemplateOrderRef(organizer, targetFolderId)

    const sourceIndex = sourceOrder.indexOf(dragging.templateName)
    if (sourceIndex < 0) return
    sourceOrder.splice(sourceIndex, 1)

    const targetIndex = targetOrder.indexOf(targetTemplateName)
    if (targetIndex < 0) {
        targetOrder.push(dragging.templateName)
    } else {
        targetOrder.splice(targetIndex, 0, dragging.templateName)
    }
    saveTemplateOrganizer()
}

const onTemplateDragEnd = () => {
    draggingTemplate.value = null
}

const isSessionAutoSubmitting = (sessionId: string) => {
    return !!autoSubmittingRecord.value[sessionId]
}

const isTemplateAutoSubmitting = (uid: number, templateName: string) => {
    return editSessions.value.some(
        session =>
            session.uid === uid &&
            session.templateName === templateName &&
            !!autoSubmittingRecord.value[session.id]
    )
}

// 获取当前模板的自动提交状态
const getCurrentAutoSubmitting = computed(() => {
    if (!activeSessionId.value) return false
    return !!autoSubmittingRecord.value[activeSessionId.value]
})

// 设置模板的自动提交状态
const setAutoSubmitting = (sessionId: string, status: boolean) => {
    const key = sessionId
    if (status) {
        autoSubmittingRecord.value[key] = true
    } else {
        delete autoSubmittingRecord.value[key]
    }
}

// 检查是否有任何模板在自动提交
const hasAnyAutoSubmitting = computed(() => {
    return Object.keys(autoSubmittingRecord.value).length > 0
})

const isVideoUploadCompleted = (video: any) => {
    if (!video) return false
    if (video.complete === true) return true
    if (video.status === 'Completed') return true
    const task = uploadStore.getUploadTask(video.id)
    return task?.status === 'Completed'
}

// 全局自动提交检查函数
const checkAutoSubmitAll = async () => {
    const sessionIds = Object.keys(autoSubmittingRecord.value)

    for (const sessionId of sessionIds) {
        const session = editSessions.value.find(item => item.id === sessionId)
        if (!session) {
            setAutoSubmitting(sessionId, false)
            continue
        }

        const user = loginUsers.value.find(u => u.uid === session.uid)
        if (!user) {
            setAutoSubmitting(sessionId, false)
            continue
        }

        const template = session.draft

        // 检查是否所有文件都已上传完成
        if (template.videos && template.videos.length > 0) {
            const allUploaded = template.videos.every(video => isVideoUploadCompleted(video))

            if (allUploaded && autoSubmittingRecord.value[sessionId]) {
                // 文件已全部上传完成，执行提交
                setAutoSubmitting(sessionId, false)
                try {
                    console.log(`自动提交触发: ${session.tabTitle}`)
                    await performTemplateSubmit(session.uid, session.templateName, template)
                } catch (error) {
                    console.error(`标签页 ${session.tabTitle} 自动提交失败:`, error)
                }
            }
        } else {
            // 没有视频文件，清除自动提交状态
            setAutoSubmitting(sessionId, false)
        }
    }

    // 如果没有模板在自动提交，停止间隔检查
    if (!hasAnyAutoSubmitting.value && autoSubmitInterval) {
        clearInterval(autoSubmitInterval)
        autoSubmitInterval = null
    }
}

// 启动全局自动提交检查
const startAutoSubmitCheck = () => {
    if (!autoSubmitInterval) {
        autoSubmitInterval = setInterval(checkAutoSubmitAll, 1000)
    }
}

// 执行模板提交
const performTemplateSubmit = async (uid: number, templateName: string, template: any) => {
    const user = loginUsers.value.find(u => u.uid === uid)
    if (!user) throw new Error('用户不存在')

    submitting.value = true
    try {
        const resp = (await uploadStore.submitTemplate(uid, template)) as any

        // 更新最后提交时间（只对当前模板）
        if (selectedUser.value?.uid === uid && currentTemplateName.value === templateName) {
            lastSubmit.value = new Date().toLocaleString()
        }

        utilsStore.showMessage(`视频${resp.bvid}提交成功 (模板: ${templateName})`, 'success')
        console.log(`视频${resp.bvid}提交成功 (模板: ${templateName})`, 'success')

        if (resp && resp.aid && utilsStore.hasSeason) {
            try {
                const old_season_id = await utilsStore.getVideoSeason(uid, resp.aid)
                let add = old_season_id && old_season_id !== 0 ? false : true

                if (template && old_season_id !== template.season_id && template.videos[0]?.cid) {
                    const new_season_id = template.season_id || 0
                    const new_section_id = template.section_id || 0
                    await utilsStore.switchSeason(
                        uid,
                        resp.aid,
                        template.videos[0]?.cid,
                        new_season_id,
                        new_section_id,
                        template.title,
                        add
                    )

                    const season_title =
                        utilsStore.seasonlist.find((s: any) => s.season_id === template.season_id)
                            ?.title || template.season_id
                    utilsStore.showMessage(`视频${resp.bvid}加入合集${season_title}`, 'success')
                    console.log(`视频${resp.bvid}加入合集${season_title}`, 'success')
                }
            } catch (error) {
                console.error('设置合集失败: ', error)
                utilsStore.showMessage(`设置合集失败: ${error}`, 'error')
            }
        }

        setTimeout(async () => {
            try {
                if (!template.aid) {
                    const userConfig = userConfigStore.configRoot?.config[uid]
                    if (userConfig && userConfig.auto_edit && newTemplateRef.value) {
                        // 新增稿件且auto_edit开启，创建编辑模板
                        await newTemplateRef.value.createTemplateFromBV(
                            uid,
                            resp.bvid,
                            resp.bvid,
                            true
                        )
                        utilsStore.showMessage('从BV号创建模板成功', 'success')
                    }
                } else {
                    if (
                        selectedUser.value?.uid === uid &&
                        currentTemplateName.value === templateName
                    ) {
                        reloadTemplateFromAV(uid, template.aid)
                    }
                }
            } catch (error) {
                utilsStore.showMessage(`${error}`, 'error')
            } finally {
                submitting.value = false
            }
        }, 500)
    } catch (error) {
        console.error('模板提交失败:', error)
        utilsStore.showMessage(`模板提交失败: ${error}`, 'error')
        submitting.value = false
    }
}
const lastSubmit = ref<string>('')

// 卡片折叠状态
const cardCollapsed = ref({
    basic: false, // 基本信息
    tags: false, // 标签设置
    description: false, // 视频描述
    videos: false, // 视频文件
    advanced: false // 高级选项
})

// 模板名编辑相关
const isEditingTemplateName = ref(false)
const editingTemplateName = ref('')
const templateNameInputRef = ref()

// 拖拽状态
const isDragOver = ref(false)

// 内容容器引用
const contentWrapperRef = ref<HTMLElement | null>(null)

// 用户配置相关
const userConfigVisible = ref(false)
const configUser = ref<any>(null)

// 分区数据
const selectedCategory = ref<any>(null)
const selectedSubCategory = ref<any>(null)
const categoryPopoverVisible = ref(false)
let generalUpdateTimer: number | null = null

const currentSession = computed(
    () => editSessions.value.find(session => session.id === activeSessionId.value) || null
)

const currentTemplate = computed(() => currentSession.value?.draft || null)

// 当前表单数据 - 直接操作模板配置
const currentForm = computed({
    get() {
        return currentSession.value?.draft || null
    },
    set(value) {
        if (!currentSession.value || !value) return
        currentSession.value.draft = value
        touchActiveSession()
        schedulePersistSessions()
    }
})

const tags = ref<string[]>([])

// 日期选择器的计算属性 - 处理时间戳转换
const dtimeDate = computed({
    get() {
        return currentForm.value?.dtime ? new Date(currentForm.value.dtime * 1000) : null
    },
    set(value: Date | null) {
        if (currentForm.value) {
            currentForm.value.dtime = value ? Math.floor(value.getTime() / 1000) : undefined
        }
    }
})

// 视频数组的计算属性 - 确保始终返回数组
const videos = computed({
    get() {
        return currentForm.value?.videos || []
    },
    set(value: any[]) {
        if (currentForm.value) {
            currentForm.value.videos = value
        }
    }
})

// 检查指定模板是否有未保存的改动
const checkTemplateHasUnsavedChanges = (uid: number, templateName: string): boolean => {
    if (!userConfigStore.configRoot?.config || !userConfigStore.configBase?.config) {
        return false
    }

    const currentUserConfig = userConfigStore.configRoot.config[uid]
    const baseUserConfig = userConfigStore.configBase.config[uid]

    if (!currentUserConfig?.templates[templateName] || !baseUserConfig?.templates[templateName]) {
        return false
    }

    const currentTemplateData = currentUserConfig.templates[templateName]
    const baseTemplateData = baseUserConfig.templates[templateName]

    return hasUnsavedChanges(baseTemplateData, currentTemplateData)
}

// 生命周期
// 监听封面变化，异步加载显示用的封面URL
watch(
    () => currentForm.value?.cover,
    async (newCover: string | undefined) => {
        if (newCover && selectedUser.value) {
            try {
                coverLoading.value = true
                const downloadedCover = await utilsStore.downloadCover(
                    selectedUser.value.uid,
                    newCover
                )
                coverDisplayUrl.value = downloadedCover || ''
            } catch (error) {
                console.error('Failed to download cover:', error)
                clearCurrentCover()
            } finally {
                coverLoading.value = false
            }
        } else {
            coverDisplayUrl.value = ''
            coverLoading.value = false
        }
    }
)

// 监听标签变化，更新表单数据
watch(
    () => tags.value,
    (newTags: string[]) => {
        if (currentForm.value) {
            currentForm.value.tag = newTags.join(',')
        }
    },
    { deep: true }
)

// 监听表单标签变化，更新标签数组
watch(
    () => currentForm.value?.tag,
    (newTag: string | undefined) => {
        const newTags = newTag ? newTag.split(',').filter(tag => tag.trim()) : []
        if (JSON.stringify(newTags) !== JSON.stringify(tags.value)) {
            tags.value = newTags
        }
    }
)

// 监听表单分区变化，更新分区选择（双向绑定）
watch(
    () => currentForm.value?.tid,
    (newTid: number | undefined) => {
        if (newTid && newTid > 0) {
            // 根据tid设置选中的分区
            setSelectedCategoryByTid(newTid)
        } else {
            // 如果没有分区信息，清空分区选择
            selectedCategory.value = null
            selectedSubCategory.value = null
        }
    }
)

// 监听用户切换，重新加载封面
watch(
    () => selectedUser.value,
    async (newUser: any) => {
        if (currentForm.value?.cover && newUser) {
            try {
                coverLoading.value = true
                const downloadedCover = await utilsStore.downloadCover(
                    newUser.uid,
                    currentForm.value.cover
                )
                coverDisplayUrl.value = downloadedCover || ''
            } catch (error) {
                console.error('Failed to download cover:', error)
                clearCurrentCover()
            } finally {
                coverLoading.value = false
            }
        } else {
            coverDisplayUrl.value = ''
            coverLoading.value = false
        }
    }
)

const getInteractiveConfirmKey = () => {
    if (!selectedUser.value?.uid) return ''
    return `${selectedUser.value.uid}`
}

const showInteractiveInfoDialog = async () => {
    await ElMessageBox.alert(
        '勾选后本视频将被投稿为互动视频，需在规定时间内完成剧情树配置，否则系统可能回收稿件。',
        '互动功能说明',
        {
            confirmButtonText: '知道了',
            type: 'warning'
        }
    )
}

const confirmInteractiveEnable = async () => {
    let dontShowAgain = false

    await ElMessageBox.confirm(
        '<div class="interactive-confirm-dialog">' +
            '<div class="interactive-confirm-dialog-text">互动视频需在规定时间内完成剧情树配置，否则系统可能回收稿件。</div>' +
            '<label class="interactive-confirm-dialog-checkbox">' +
            '<input id="interactive-dont-show-again" type="checkbox" />' +
            '<span>以后不再显示提示</span>' +
            '</label>' +
            '</div>',
        '确认',
        {
            confirmButtonText: '是，我已知晓',
            cancelButtonText: '否',
            type: 'warning',
            dangerouslyUseHTMLString: true,
            beforeClose: (action, _instance, done) => {
                void _instance
                if (action === 'confirm') {
                    const checkbox = document.getElementById(
                        'interactive-dont-show-again'
                    ) as HTMLInputElement | null
                    dontShowAgain = Boolean(checkbox?.checked)
                }
                done()
            }
        }
    )

    return dontShowAgain
}

watch(
    () => currentForm.value?.interactive,
    async (newValue, oldValue) => {
        if (!currentForm.value || interactiveDialogOpening.value || templateLoading.value) return
        if (newValue !== 1 || oldValue === 1) return

        const key = getInteractiveConfirmKey()
        if (!key || interactiveConfirmShown.value[key]) return

        interactiveDialogOpening.value = true
        try {
            const dontShowAgain = await confirmInteractiveEnable()
            if (dontShowAgain) {
                interactiveConfirmShown.value[key] = true
            }
        } catch {
            currentForm.value.interactive = 0
        } finally {
            interactiveDialogOpening.value = false
        }
    }
)

let keyboardCleanup: (() => void) | null = null

const forwardConsole = (fnName: keyof Console, logger: (level: string, ...args: any[]) => void) => {
    const original = console[fnName] as (...args: any[]) => void
    ;(console as any)[fnName] = (...args: any[]) => {
        original(...args)
        logger(fnName as string, ...args)
    }
}

onMounted(async () => {
    loadTemplateOrganizer()
    reloadLocalUiSettings()
    loadSidebarWidth()
    await initializeData()
    await setupDragAndDrop()
    keyboardCleanup = await setupKeyboardShortcuts()

    forwardConsole('log', utilsStore.log)
    forwardConsole('error', utilsStore.log)
    forwardConsole('warn', utilsStore.log)

    // 禁用右键菜单刷新
    document.addEventListener('contextmenu', (event: MouseEvent) => {
        event.preventDefault()
    })

    const onResize = () => {
        sidebarWidth.value = clampSidebarWidth(sidebarWidth.value)
    }
    window.addEventListener('resize', onResize)
    sidebarResizeCleanup = () => window.removeEventListener('resize', onResize)
})

// 在组件卸载时清理
onUnmounted(() => {
    if (keyboardCleanup) {
        keyboardCleanup()
    }

    // 清理自动提交间隔检查
    if (autoSubmitInterval) {
        clearInterval(autoSubmitInterval)
        autoSubmitInterval = null
    }

    if (generalUpdateTimer) {
        clearInterval(generalUpdateTimer)
        generalUpdateTimer = null
    }

    if (sessionPersistTimer) {
        clearTimeout(sessionPersistTimer)
        sessionPersistTimer = null
    }

    if (sidebarResizeCleanup) {
        sidebarResizeCleanup()
        sidebarResizeCleanup = null
    }

    // 清理所有自动提交状态
    autoSubmittingRecord.value = {}
})

// 初始化数据
const initializeData = async () => {
    try {
        currentVer.value = (await utilsStore.getCurrentVersion()) as string
        // 获取登录用户
        await authStore.getLoginUsers()

        // 构建用户模板列表
        if (loginUsers.value.length > 0) {
            await utilsStore.initTypeList(loginUsers.value[0].uid)
            await utilsStore.initTopicList(loginUsers.value[0].uid)
            await userConfigStore.buildUserTemplates(loginUsers.value)
            syncTemplateOrganizer()
            await uploadStore.getUploadQueue()
            if (!generalUpdateTimer) {
                generalUpdateTimer = setInterval(() => {
                    if (authStore.loginUsers.length > 0) {
                        uploadStore.getUploadQueue()
                    }
                    for (const task of uploadStore.uploadQueue) {
                        const templateName = task.template
                        const uid = task.user.uid

                        const applyTaskToVideo = (video: any) => {
                            if (!video) return
                            video.status = task.status || video.status
                            video.errorMessage = task.error_message || ''
                            video.totalSize = task.total_size || video.totalSize || 0
                            video.speed = task.speed || 0
                            video.progress = task.progress || video.progress || 0
                            if (task.video?.filename) video.filename = task.video.filename
                            if (typeof task.video?.cid === 'number') video.cid = task.video.cid
                            if (task.finished_at) video.finished_at = task.finished_at
                            if (task.status === 'Completed') {
                                video.complete = true
                            }
                        }

                        const configVideos =
                            userConfigStore.configRoot?.config[uid]?.templates[templateName]
                                ?.videos || []
                        const configVideo = configVideos.find(v => v.id === task.video?.id)
                        if (configVideo) {
                            applyTaskToVideo(configVideo)
                        }

                        for (const session of editSessions.value) {
                            if (session.uid !== uid || session.templateName !== templateName) continue
                            const sessionVideo = session.draft?.videos?.find(v => v.id === task.video?.id)
                            if (sessionVideo) {
                                applyTaskToVideo(sessionVideo)
                            }
                        }
                    }
                }, 666) // 更新上传队列
            }
        }

        setTimeout(async () => {
            const restoredSessions = await restoreTemplateSessions()
            if (!restoredSessions) {
                await restoreTemplateSelection()
            }
            restoreCardCollapsedState()
        }, 100)
    } catch (error) {
        console.error('初始化数据失败: ', error)
        utilsStore.showMessage(`'初始化数据失败: ${error}'`, 'error')
    }
}

watch(
    () =>
        userTemplates.value.map(userTemplate => ({
            uid: userTemplate.user.uid,
            templates: userTemplate.templates.map(template => template.name).sort()
        })),
    () => {
        syncTemplateOrganizer()
    },
    { deep: true }
)

watch(
    () => editSessions.value,
    () => {
        schedulePersistSessions()
    },
    { deep: true }
)

watch(
    () => activeSessionId.value,
    () => {
        schedulePersistSessions()
    }
)

const hasUnsavedChanges = (
    baseTemplateData: TemplateConfig,
    currentTemplateData: TemplateConfig
) => {
    // 比较关键字段
    const fieldsToCompare = [
        'title',
        'cover',
        'copyright',
        'source',
        'tid',
        'desc',
        'dynamic',
        'tag',
        'dtime',
        'open_subtitle',
        'interactive',
        'mission_id',
        'topic_id',
        'season_id',
        'section_id',
        'dolby',
        'lossless_music',
        'no_reprint',
        'open_elec',
        'up_selection_reply',
        'up_close_reply',
        'up_close_danmu',
        'is_only_self',
        'watermark'
    ]

    for (const field of fieldsToCompare) {
        const currentValue = (currentTemplateData as any)[field]
        const baseValue = (baseTemplateData as any)[field]

        // 处理 undefined/null/空字符串 的情况
        if (
            (currentValue === undefined || currentValue === null || currentValue === '') &&
            (baseValue === undefined || baseValue === null || baseValue === '')
        ) {
            continue
        }

        if (JSON.stringify(currentValue) !== JSON.stringify(baseValue)) {
            // console.log(field, '有改动')
            // console.log('current: ', JSON.stringify(currentValue), 'vs', JSON.stringify(baseValue))
            return true
        }
    }

    // 特别比较 videos 数组
    const currentVideos = currentTemplateData.videos || []
    const baseVideos = baseTemplateData.videos || []

    if (currentVideos.length !== baseVideos.length) {
        return true
    }

    // 比较视频的关键字段
    for (let i = 0; i < currentVideos.length; i++) {
        const currentVideo = currentVideos[i]
        const baseVideo = baseVideos[i]

        const videoFieldsToCompare = ['title', 'filename', 'desc', 'path', 'cid']
        for (const field of videoFieldsToCompare) {
            if (
                JSON.stringify((currentVideo as any)[field]) !==
                JSON.stringify((baseVideo as any)[field])
            ) {
                return true
            }
        }
    }

    return false
}

// 设置拖拽功能
const setupDragAndDrop = async () => {
    try {
        // 监听文件拖拽事件
        await listen('tauri://drag-drop', async event => {
            const videos = event.payload as string[]
            isDragOver.value = false
            if (templateLoading.value) {
                utilsStore.showMessage('模板加载中', 'warning')
                return
            }
            await handleDroppedFiles(videos)
        })

        // 监听拖拽悬停事件
        await listen('tauri://drag-over', event => {
            if (!isDragOver.value) console.log('文件拖拽悬停:', event.payload, '，忽略后续日志')
            isDragOver.value = true
        })

        // 监听拖拽取消事件
        await listen('tauri://drag-leave', () => {
            console.log('文件拖拽取消')
            isDragOver.value = false
        })
    } catch (error) {
        console.error('设置拖拽功能失败: ', error)
        utilsStore.showMessage(`'设置拖拽功能失败: ${error}'`, 'error')
    }
}

// 设置键盘快捷键
const setupKeyboardShortcuts = async () => {
    const handleKeydown = (event: KeyboardEvent) => {
        // 禁用 F5 刷新
        if (!event.ctrlKey && event.key === 'F5') {
            event.preventDefault()
            return
        }

        // Ctrl+F5 刷新页面
        if (event.ctrlKey && event.key === 'F5') {
            event.preventDefault()
            window.location.reload()
            return
        }

        if (event.ctrlKey && event.key === 'r') {
            event.preventDefault()
            if (selectedUser.value && currentTemplateName.value) {
                resetTemplate()
            }
            return
        }

        // Ctrl+S 保存模板
        if (event.ctrlKey && event.key === 's') {
            event.preventDefault()
            if (selectedUser.value && currentTemplateName.value) {
                saveTemplate()
            }
            return
        }

        // Ctrl+W 关闭当前标签页
        if (event.ctrlKey && event.key.toLowerCase() === 'w') {
            event.preventDefault()
            if (activeSessionId.value) {
                closeSessionTab(activeSessionId.value)
            }
        }
    }

    document.addEventListener('keydown', handleKeydown)

    // 返回清理函数
    return () => {
        document.removeEventListener('keydown', handleKeydown)
    }
}

// 切换卡片折叠状态
const toggleCardCollapsed = (cardKey: keyof typeof cardCollapsed.value) => {
    cardCollapsed.value[cardKey] = !cardCollapsed.value[cardKey]
    // 保存折叠状态到localStorage
    saveCardCollapsedState()
}

// 模板选择记忆功能
const TEMPLATE_SELECTION_KEY = 'last-selected-template'
const CARD_COLLAPSED_KEY = 'card-collapsed-state'

// 是否正在恢复模板选择（避免递归保存）
const isRestoringTemplate = ref(false)

// 保存模板选择到localStorage
const saveTemplateSelection = (userUid: number, templateName: string) => {
    // 如果正在恢复模板，不保存（避免递归）
    if (isRestoringTemplate.value) return

    try {
        const selection = {
            userUid,
            templateName,
            timestamp: Date.now()
        }
        localStorage.setItem(TEMPLATE_SELECTION_KEY, JSON.stringify(selection))
    } catch (error) {
        console.error('保存模板选择失败:', error)
    }
}

// 保存卡片折叠状态
const saveCardCollapsedState = () => {
    try {
        localStorage.setItem(CARD_COLLAPSED_KEY, JSON.stringify(cardCollapsed.value))
    } catch (error) {
        console.error('保存卡片折叠状态失败:', error)
    }
}

// 恢复卡片折叠状态
const restoreCardCollapsedState = () => {
    try {
        const saved = localStorage.getItem(CARD_COLLAPSED_KEY)
        if (saved) {
            const savedState = JSON.parse(saved)
            Object.assign(cardCollapsed.value, savedState)
        }
    } catch (error) {
        console.error('恢复卡片折叠状态失败:', error)
    }
}

const restoreTemplateSessions = async () => {
    try {
        const raw = localStorage.getItem(TEMPLATE_SESSIONS_KEY)
        if (!raw) return false
        const parsed = JSON.parse(raw) as {
            activeSessionId?: string
            sessions?: TemplateEditSession[]
        }
        if (!parsed?.sessions?.length) return false

        const restored: TemplateEditSession[] = []
        for (const session of parsed.sessions) {
            const user = loginUsers.value.find(item => item.uid === session.uid)
            if (!user || user.expired) continue
            const template = userConfigStore.getUserTemplate(session.uid, session.templateName)
            if (!template) continue
            restored.push({
                ...session,
                username: user.username || session.username,
                draft: cloneTemplateConfig(session.draft || template),
                updatedAt: Date.now()
            })
        }
        if (!restored.length) return false

        editSessions.value = restored.slice(0, getTemplateTabMax())
        const activeId = parsed.activeSessionId || editSessions.value[0].id
        const existsActive = editSessions.value.some(item => item.id === activeId)
        await setActiveSession(existsActive ? activeId : editSessions.value[0].id)
        schedulePersistSessions()
        utilsStore.showMessage('已恢复上次编辑标签页', 'success')
        return true
    } catch (error) {
        console.error('恢复编辑会话失败:', error)
        return false
    }
}

// 从localStorage恢复模板选择
const restoreTemplateSelection = async () => {
    try {
        const saved = localStorage.getItem(TEMPLATE_SELECTION_KEY)
        if (!saved) return

        const selection = JSON.parse(saved)
        const { userUid, templateName, timestamp } = selection

        // 检查数据有效性（超过30天的记录自动失效）
        const thirtyDaysAgo = Date.now() - 30 * 24 * 60 * 60 * 1000
        if (timestamp && timestamp < thirtyDaysAgo) {
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        // 检查用户是否仍然登录
        const targetUser = loginUsers.value.find(user => user.uid === userUid)
        if (!targetUser) {
            // 用户已不存在，清除记录
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        if (targetUser.expired) {
            // 过期账号不恢复历史模板选择
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        // 检查模板是否仍然存在
        const userTemplate = userTemplates.value.find(ut => ut.user.uid === userUid)
        const template = userTemplate?.templates.find(t => t.name === templateName)
        if (!template) {
            // 模板已不存在，清除记录
            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            return
        }

        // 确保用户是展开状态
        if (userTemplate && !userTemplate.expanded) {
            toggleUserExpanded(userUid)
        }

        // 设置恢复状态标志
        isRestoringTemplate.value = true

        // 自动选择模板
        await selectTemplate(targetUser, templateName)

        // 恢复完成后重置标志
        isRestoringTemplate.value = false

        console.log(`自动恢复模板选择: ${targetUser.username} - ${templateName}`)
        utilsStore.showMessage(`已恢复上次选择的模板: ${templateName}`, 'success')
    } catch (error) {
        console.error('恢复模板选择失败:', error)
        // 如果恢复失败，清除无效的存储数据
        localStorage.removeItem(TEMPLATE_SELECTION_KEY)
    } finally {
        // 确保标志被重置
        isRestoringTemplate.value = false
    }
}

// 清空卡片内容
const clearCardContent = async (cardType: 'basic' | 'tags' | 'description' | 'advanced') => {
    if (!currentForm.value) {
        utilsStore.showMessage('请先选择模板', 'warning')
        return
    }

    // 如果正在加载模板，禁止清空
    if (templateLoading.value) {
        utilsStore.showMessage('模板正在加载中，请稍后再试', 'warning')
        return
    }

    try {
        // 确认清空
        await ElMessageBox.confirm(
            `确定要清空"${getCardDisplayName(cardType)}"的所有内容吗？`,
            '确认清空',
            {
                confirmButtonText: '确定',
                cancelButtonText: '取消',
                type: 'warning'
            }
        )

        // 根据卡片类型清空相应内容
        switch (cardType) {
            case 'basic':
                currentForm.value.title = ''
                currentForm.value.cover = ''
                currentForm.value.tid = 0
                currentForm.value.copyright = 1
                currentForm.value.source = ''
                // 同步清空分区选择状态
                selectedCategory.value = null
                selectedSubCategory.value = null
                // 清空封面显示
                coverDisplayUrl.value = ''
                break

            case 'tags':
                currentForm.value.tag = ''
                // 同步清空标签数组
                tags.value = []
                // 通过组件引用清空TagView的状态
                tagViewRef.value?.clearTags()
                break

            case 'description':
                currentForm.value.desc = ''
                currentForm.value.dynamic = ''
                break

            case 'advanced':
                currentForm.value.watermark =
                    userConfigStore?.configRoot?.config[selectedUser.value.uid]?.watermark || 0
                currentForm.value.dtime = undefined
                currentForm.value.interactive = 0
                currentForm.value.dolby = 0
                currentForm.value.lossless_music = 0
                currentForm.value.no_reprint = 0
                currentForm.value.open_elec = 0
                currentForm.value.up_selection_reply = 0
                currentForm.value.up_close_reply = 0
                currentForm.value.up_close_danmu = 0
                currentForm.value.atomic_int = 0
                currentForm.value.is_only_self = 0
                break
        }

        utilsStore.showMessage(`已清空"${getCardDisplayName(cardType)}"的内容`, 'success')
    } catch (error) {
        // 用户取消了操作
    }
}

// 获取卡片显示名称
const getCardDisplayName = (cardType: string): string => {
    const cardNames: Record<string, string> = {
        basic: '基本信息',
        tags: '标签设置',
        description: '视频描述',
        videos: '视频文件',
        advanced: '高级选项'
    }
    return cardNames[cardType] || cardType
}

const ensureTitleFromFirstVideo = (videoTitle: string) => {
    if (!currentForm.value) return

    const currentTitle = (currentForm.value.title || '').trim()
    if (currentTitle) return

    const importedVideoTitle = (videoTitle || '').trim()
    if (importedVideoTitle) {
        currentForm.value.title = importedVideoTitle
    }
}

const addVideoToCurrentForm = async (videoPath: string, customTitle?: string) => {
    // 从路径中提取文件名
    const videoBaseName = videoPath.split(/[/\\]/).pop() || videoPath
    const videoNameWOExtension = videoBaseName.replace(/\.[^/.]+$/, '').slice(0, 80)
    const finalTitle = (customTitle || videoNameWOExtension).slice(0, 80)
    const videoExt = videoBaseName.split('.').pop()?.toLowerCase() || ''

    const extFilter = [
        'mp4',
        'flv',
        'avi',
        'wmv',
        'mov',
        'webm',
        'mpeg4',
        'ts',
        'mpg',
        'rm',
        'rmvb',
        'mkv',
        'm4v'
    ]

    if (videoExt && !extFilter.includes(videoExt)) {
        return 0 // 不支持的格式，跳过添加
    }

    // 检查文件是否已经存在
    if (!currentForm.value) {
        return 0 // 没有当前模板，跳过添加
    }

    const existingFile = currentForm.value.videos.find(
        f => f.path === videoPath || finalTitle === f.title
    )
    if (existingFile) {
        return 0 // 跳过已存在的文件
    }

    const currentAddedVideos = currentForm.value.videos.filter(video => {
        return (
            (video.finished_at && video.finished_at > 0) || (video.path && video.path.trim() !== '')
        )
    })

    // 检查是否超过100个视频的限制
    if (currentAddedVideos.length >= 100) {
        utilsStore.showMessage('单次提交最大限制100个视频文件，无法添加更多视频', 'error')
        return 0
    }

    let fileMtime = 0
    try {
        const fileInfo = await stat(videoPath)
        fileMtime = fileInfo.mtime ? fileInfo.mtime.getTime() : 0
    } catch (error) {
        console.warn('读取视频文件修改时间失败:', error)
    }

    // 添加到currentForm.videos
    const videoId = uuidv4()
    currentForm.value.videos.push({
        id: videoId,
        filename: videoBaseName, // 使用完整的文件路径
        title: finalTitle, // 去除扩展名作为标题或使用自定义标题
        desc: '',
        path: videoPath, // 保存完整路径
        complete: false,
        mtime: fileMtime
    })

    // 标题为空时，自动使用本次导入的第一个视频文件名（去扩展名）作为标题
    ensureTitleFromFirstVideo(finalTitle)

    // 检查是否启用自动添加到上传队列
    if (userConfigStore.configRoot?.auto_upload && selectedUser.value) {
        try {
            // 自动创建上传任务
            await uploadStore.createUploadTask(
                selectedUser.value.uid,
                currentTemplateName.value,
                currentForm.value.videos
            )
            console.log(`自动添加文件到上传队列: ${videoBaseName}`)

            // 如果同时启用自动开始，则自动开始任务
            if (userConfigStore.configRoot?.auto_start) {
                // 延迟一下让任务先添加到队列
                setTimeout(async () => {
                    try {
                        await autoStartWaitingTasks()
                    } catch (error) {
                        console.error('自动开始任务失败:', error)
                    }
                }, 500)
            }
        } catch (error) {
            console.error('自动添加到上传队列失败:', error)
        }
    }
    return 1
}

// 处理拖拽文件
const handleDroppedFiles = async (videoFiles: any) => {
    // 检查是否有选中的用户和模板
    if (!selectedUser.value || !currentTemplateName.value) {
        utilsStore.showMessage('请先选择用户和模板后再拖拽文件', 'warning')
        return
    }

    // 添加视频文件到当前模板
    let addedCount = 0
    templateLoading.value = true
    for (const videoPath of videoFiles.paths) {
        addedCount += await addVideoToCurrentForm(videoPath)
    }
    templateLoading.value = false

    if (addedCount > 0) {
        utilsStore.showMessage(`成功添加 ${addedCount} 个视频文件`, 'success')
    } else {
        utilsStore.showMessage('所有文件都已存在，未添加新文件', 'info')
    }
}

// 处理登录成功
const handleLoginSuccess = async () => {
    showLoginDialog.value = false
    utilsStore.showMessage('登录成功', 'success')

    await userConfigStore.saveConfig()
    // 刷新所有数据
    await refreshAllData()
}

// 处理登录对话框关闭
const handleLoginDialogClose = async (done: () => void) => {
    if (loginLoading.value) {
        utilsStore.showMessage('登录过程中无法取消', 'warning')
        return
    }

    try {
        await ElMessageBox.confirm('确定要取消登录吗？', '提示', {
            confirmButtonText: '确定',
            cancelButtonText: '继续登录',
            type: 'warning'
        })
        done()
    } catch (error) {
        // 用户点击了取消，不关闭对话框
    }
}

// 切换用户展开状态
const toggleUserExpanded = (userUid: number) => {
    userConfigStore.toggleUserExpanded(userUid)
}

// 处理用户展开按钮点击 - 在模板加载时禁用
const handleUserExpansion = (userUid: number) => {
    const targetUser = loginUsers.value.find(user => user.uid === userUid)
    if (targetUser?.expired) {
        showLoginDialog.value = true
        utilsStore.showMessage('该用户 Cookie 已过期，请重新登录', 'warning')
        return
    }

    if (!templateLoading.value) {
        toggleUserExpanded(userUid)
    }
}

// 处理模板选择点击 - 在模板加载时禁用
const handleTemplateSelection = (user: any, templateName: string, event?: MouseEvent) => {
    if (event) {
        const target = event.target as HTMLElement | null
        if (
            target?.closest('.template-menu-btn') ||
            target?.closest('.el-dropdown') ||
            target?.closest('.el-dropdown-menu')
        ) {
            return
        }
    }

    if (user?.expired) {
        showLoginDialog.value = true
        utilsStore.showMessage('该用户 Cookie 已过期，请重新登录', 'warning')
        return
    }

    if (!templateLoading.value) {
        selectTemplate(user, templateName)
    }
}

// 处理模板名编辑点击 - 在模板加载时禁用
const handleTemplateNameEdit = () => {
    if (!templateLoading.value) {
        startEditTemplateName()
    }
}

// 处理封面选择点击 - 在模板加载时禁用
const handleCoverSelection = () => {
    if (!templateLoading.value) {
        selectCoverWithTauri()
    }
}

const getSessionTitle = (uid: number, templateName: string) => {
    const count = editSessions.value.filter(
        session => session.uid === uid && session.templateName === templateName
    ).length
    return `${templateName} #${count + 1}`
}

const setActiveSession = async (sessionId: string) => {
    const session = editSessions.value.find(item => item.id === sessionId)
    if (!session) return
    activeSessionId.value = session.id
    selectedUser.value =
        loginUsers.value.find(user => user.uid === session.uid) || {
            uid: session.uid,
            username: session.username
        }
    currentTemplateName.value = session.templateName
    touchActiveSession()
    await loadTemplate()
    saveTemplateSelection(session.uid, session.templateName)
    schedulePersistSessions()
}

const openTemplateSession = async (user: any, templateName: string) => {
    if (editSessions.value.length >= getTemplateTabMax()) {
        utilsStore.showMessage(`最多可打开 ${getTemplateTabMax()} 个编辑标签页`, 'warning')
        return
    }

    const template = userConfigStore.getUserTemplate(user.uid, templateName)
    const session: TemplateEditSession = {
        id: uuidv4(),
        uid: user.uid,
        username: user.username || '',
        templateName,
        tabTitle: getSessionTitle(user.uid, templateName),
        createdAt: Date.now(),
        updatedAt: Date.now(),
        draft: cloneTemplateConfig(template)
    }
    editSessions.value.push(session)
    lastSubmit.value = ''
    await setActiveSession(session.id)

    // 滚动到顶部
    nextTick(() => {
        if (contentWrapperRef.value) {
            contentWrapperRef.value.scrollTop = 0
        }
    })

    const aid = session.draft?.aid
    setTimeout(async () => {
        if (!aid) return
        try {
            const currentSession = editSessions.value.find(item => item.id === session.id)
            if (!currentSession) return
            const newTemplate = await getNewTemplateFromAv(user.uid, aid)
            const savedTemplate = userConfigStore.getUserTemplate(user.uid, templateName)
            if (savedTemplate && hasUnsavedChanges(savedTemplate, newTemplate)) {
                await ElMessageBox.confirm(
                    `检测到本地模板内容与bilibili不一致，是否刷新？（此操作会丢失当前标签页未保存的更改）`,
                    '',
                    {
                        confirmButtonText: '刷新并继续',
                        cancelButtonText: '不刷新，仅显示当前',
                        type: 'info'
                    }
                )
                currentSession.draft = cloneTemplateConfig(newTemplate)
                if (activeSessionId.value === session.id) {
                    await loadTemplate()
                }
                schedulePersistSessions()
            }
        } catch (error) {
            console.error('自动刷新模板数据失败:', error)
        }
    }, 666)
}

const closeSessionTab = async (sessionId: string) => {
    const index = editSessions.value.findIndex(item => item.id === sessionId)
    if (index < 0) return

    const closing = editSessions.value[index]
    const savedTemplate = userConfigStore.getUserTemplate(closing.uid, closing.templateName)
    if (savedTemplate && hasUnsavedChanges(savedTemplate, closing.draft)) {
        try {
            await ElMessageBox.confirm(
                `标签页 "${closing.tabTitle}" 有未保存修改，确定关闭吗？`,
                '关闭标签页',
                {
                    confirmButtonText: '关闭',
                    cancelButtonText: '取消',
                    type: 'warning'
                }
            )
        } catch {
            return
        }
    }

    editSessions.value.splice(index, 1)
    setAutoSubmitting(sessionId, false)
    if (activeSessionId.value === sessionId) {
        const fallback = editSessions.value[Math.max(0, index - 1)] || editSessions.value[0]
        if (fallback) {
            await setActiveSession(fallback.id)
        } else {
            activeSessionId.value = ''
            selectedUser.value = null
            currentTemplateName.value = ''
            tags.value = []
            selectedCategory.value = null
            selectedSubCategory.value = null
        }
    }
    schedulePersistSessions()
}

const handleSessionTabClick = async (pane: any) => {
    if (!pane?.props?.name) return
    if (pane.props.name === activeSessionId.value) return
    await setActiveSession(String(pane.props.name))
}

const handleTabLabelMouseup = async (event: MouseEvent, sessionId: string) => {
    if (event.button !== 1) return
    event.preventDefault()
    event.stopPropagation()
    await closeSessionTab(sessionId)
}

// 选择模板（打开新标签页）
const selectTemplate = async (user: any, templateName: string) => {
    if (templateLoading.value) return

    templateLoading.value = true
    try {
        await openTemplateSession(user, templateName)
        console.log(`已打开模板标签页: ${user.username} - ${templateName}`)
    } catch (error) {
        console.error('打开模板标签页失败:', error)
        utilsStore.showMessage(`打开模板标签页失败: ${error}`, 'error')
    } finally {
        templateLoading.value = false
    }
}

const resetTemplate = async () => {
    if (!selectedUser.value || !currentTemplateName.value) {
        utilsStore.showMessage('请先选择用户和模板', 'warning')
        return
    }

    // 如果正在加载模板，禁止重置
    if (templateLoading.value) {
        utilsStore.showMessage('模板正在加载中，请稍后再试', 'warning')
        return
    }

    // 确认重置
    try {
        await ElMessageBox.confirm('确定要清除所有未保存的更改吗?', '', {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            type: 'warning'
        })

        templateLoading.value = true
        try {
            currentForm.value =
                JSON.parse(
                    JSON.stringify(
                        userConfigStore.configBase?.config[selectedUser.value.uid]?.templates[
                            currentTemplateName.value
                        ]
                    )
                ) || userConfigStore.createDefaultTemplate()
            utilsStore.showMessage('模板已重置', 'success')
        } finally {
            templateLoading.value = false
        }
    } catch (error) {
        // 用户取消了重置
        console.log('重置操作已取消')
    }
}

const getNewTemplateFromAv = async (userUid: number, aid: number) => {
    try {
        const newTemplate = (await utilsStore.getVideoDetail(userUid, aid.toString())) as any

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

        newTemplate.watermark = currentForm.value?.watermark
        return newTemplate
    } catch (error) {
        console.error('获取新模板失败: ', error)
        throw error
    }
}

const reloadTemplateFromAV = async (userUid: number, aid: number) => {
    // 如果正在加载模板，禁止重新加载
    if (templateLoading.value) {
        return
    }

    if (!selectedUser.value || selectedUser.value.uid !== userUid) {
        return
    }

    if (!currentForm.value || currentForm.value.aid !== aid) {
        return
    }

    templateLoading.value = true
    try {
        const newTemplate = await getNewTemplateFromAv(userUid, aid)
        currentForm.value = newTemplate
        utilsStore.showMessage('模板数据已刷新', 'success')
    } catch (error) {
        console.error('刷新失败: ', error)
        utilsStore.showMessage(`刷新失败: ${error}`, 'error')
        throw error
    } finally {
        templateLoading.value = false
    }
}

// 加载模板数据到表单
const loadTemplate = async () => {
    try {
        // 如果当前标签页草稿为空，则填充默认模板配置
        if (!currentTemplate.value) {
            if (currentSession.value) {
                currentSession.value.draft = cloneTemplateConfig()
                touchActiveSession()
                schedulePersistSessions()
            }

            // 清空标签
            tags.value = []

            // 清空分区选择
            selectedCategory.value = null
            selectedSubCategory.value = null

            // 等待所有更新完成
            await nextTick()

            return
        }

        const template = currentTemplate.value

        // 解析标签
        tags.value = template.tag ? template.tag.split(',').filter(tag => tag.trim()) : []

        // 设置选中的分区
        if (template.tid) {
            setSelectedCategoryByTid(template.tid)
        } else {
            // 如果没有分区信息，清空分区选择
            selectedCategory.value = null
            selectedSubCategory.value = null
        }

        // 等待所有更新完成
        await nextTick()

        // 模板数据已直接操作，无需保存基础状态
    } catch (error) {
        console.error('加载模板失败:', error)
        utilsStore.showMessage(`加载模板失败: ${error}`, 'error')
    }
}

// 处理模板命令
const handleTemplateCommand = async (
    command: string,
    user: any,
    template: any,
    folderId?: string
) => {
    switch (command) {
        case 'duplicate':
            try {
                const newName = `${template.name}_副本`
                await userConfigStore.duplicateUserTemplate(user.uid, template.name, newName)
                const organizer = getUserOrganizer(user.uid)
                if (folderId) {
                    const folder = organizer.folders.find(item => item.id === folderId)
                    if (folder) {
                        if (!folder.order.includes(newName)) {
                            folder.order.push(newName)
                        }
                    } else {
                        ensureTemplateInOrganizer(user.uid, newName)
                    }
                } else {
                    ensureTemplateInOrganizer(user.uid, newName)
                }
                saveTemplateOrganizer()
                utilsStore.showMessage('模板复制成功', 'success')
            } catch (error) {
                console.error('复制模板失败: ', error)
                utilsStore.showMessage(`'复制模板失败: ${error}'`, 'error')
            }
            break

        case 'rename':
            try {
                const { value: newName } = await ElMessageBox.prompt(
                    '请输入新的模板名称',
                    '重命名模板',
                    {
                        confirmButtonText: '确定',
                        cancelButtonText: '取消',
                        inputPlaceholder: '请输入模板名称',
                        inputValue: template.name,
                        inputValidator: (value: string) => {
                            if (!value || !value.trim()) {
                                return '模板名称不能为空'
                            }
                            if (value.trim() === template.name) {
                                return '新名称不能与原名称相同'
                            }
                            return true
                        }
                    }
                )

                const trimmedName = newName.trim()

                // 检查是否已存在同名模板
                const existingTemplate = userConfigStore.getUserTemplate(user.uid, trimmedName)
                if (existingTemplate) {
                    utilsStore.showMessage('该名称的模板已存在，请使用其他名称', 'error')
                    return
                }

                // 获取原模板配置
                const originalTemplate = userConfigStore.getUserTemplate(user.uid, template.name)
                if (!originalTemplate) {
                    utilsStore.showMessage('原模板不存在', 'error')
                    return
                }

                // 先添加新模板
                await userConfigStore.addUserTemplate(user.uid, trimmedName, originalTemplate)

                // 再删除原模板
                await userConfigStore.removeUserTemplate(user.uid, template.name)
                const organizer = getUserOrganizer(user.uid)
                const sourceFolder = organizer.folders.find(folder =>
                    folder.order.includes(template.name)
                )
                removeTemplateFromOrganizer(user.uid, template.name)
                removeTemplateFromOrganizer(user.uid, trimmedName)
                if (sourceFolder) {
                    sourceFolder.order.push(trimmedName)
                } else {
                    organizer.ungroupedOrder.push(trimmedName)
                }
                editSessions.value.forEach(session => {
                    if (session.uid === user.uid && session.templateName === template.name) {
                        session.templateName = trimmedName
                        session.tabTitle = session.tabTitle.replace(template.name, trimmedName)
                        if (session.id === activeSessionId.value) {
                            currentTemplateName.value = trimmedName
                        }
                    }
                })
                saveTemplateOrganizer()
                schedulePersistSessions()

                // 更新当前选择
                if (
                    selectedUser.value?.uid === user.uid &&
                    currentTemplateName.value === template.name
                ) {
                    currentTemplateName.value = trimmedName
                    // 更新localStorage中的模板选择记录
                    saveTemplateSelection(user.uid, trimmedName)
                }

                utilsStore.showMessage('模板重命名成功', 'success')
            } catch (error) {
                if (error !== 'cancel') {
                    console.error('重命名模板失败: ', error)
                    utilsStore.showMessage(`'重命名模板失败: ${error}'`, 'error')
                }
            }
            break

        case 'delete':
            try {
                const template_name = template.name || currentTemplateName.value
                await ElMessageBox.confirm(`确定要删除模板"${template_name}"吗？`, '确认删除', {
                    confirmButtonText: '确定',
                    cancelButtonText: '取消',
                    type: 'warning'
                })

                await userConfigStore.removeUserTemplate(user.uid, template_name)
                removeTemplateFromOrganizer(user.uid, template_name)
                saveTemplateOrganizer()
                const deleteSessionIds = editSessions.value
                    .filter(item => item.uid === user.uid && item.templateName === template_name)
                    .map(item => item.id)
                for (const sessionId of deleteSessionIds) {
                    await closeSessionTab(sessionId)
                }

                // 如果删除的是当前选中的模板，清空选择
                if (
                    selectedUser.value?.uid === user.uid &&
                    currentTemplateName.value === template_name
                ) {
                    currentTemplateName.value = ''
                    selectedUser.value = null
                    // 清除localStorage中的模板选择记录
                    localStorage.removeItem(TEMPLATE_SELECTION_KEY)
                }

                utilsStore.showMessage('模板删除成功', 'success')
            } catch (error) {
                if (error !== 'cancel') {
                    console.error('删除模板失败: ', error)
                    utilsStore.showMessage(`'删除模板失败: ${error}'`, 'error')
                }
            }
            break

        case 'move':
            openMoveDialog(user.uid, template.name, folderId)
            break

        case 'move_root':
            moveTemplateToRoot(user.uid, template.name)
            break

        case 'set_cover':
            await selectTemplateCover(user.uid, template.name)
            break

        case 'clear_cover':
            clearTemplateCoverPath(user.uid, template.name)
            utilsStore.showMessage('已清除模板封面', 'success')
            break

        case 'move_up':
            moveTemplatePosition(user.uid, template.name, 'up')
            break

        case 'move_down':
            moveTemplatePosition(user.uid, template.name, 'down')
            break
    }
}

// 处理模板创建成功事件
const handleTemplateCreated = async (userUid: number, templateName: string) => {
    if (getCurrentAutoSubmitting.value) {
        return
    }

    // 自动选择新创建的模板
    const targetUser = loginUsers.value.find(user => user.uid === userUid)
    if (targetUser) {
        ensureTemplateInOrganizer(userUid, templateName)
        saveTemplateOrganizer()
        console.log(
            `模板创建事件: uid=${userUid}, template=${templateName}, organizer=${JSON.stringify(
                getUserOrganizer(userUid)
            )}`
        )
        templateLoading.value = true
        await openTemplateSession(targetUser, templateName)
        templateLoading.value = false
    }
}

// 保存模板
const saveTemplate = async () => {
    if (!selectedUser.value || !currentTemplateName.value || !currentTemplate.value) {
        utilsStore.showMessage('请先选择模板', 'error')
        return
    }

    try {
        // 直接保存当前模板配置
        await userConfigStore.updateUserTemplate(
            selectedUser.value.uid,
            currentTemplateName.value,
            currentTemplate.value
        )

        // 模板数据已直接操作并保存，无需额外状态管理
    } catch (error) {
        console.error('保存模板失败: ', error)
        utilsStore.showMessage(`'保存模板失败: ${error}'`, 'error')
    }
}

// 分区选择相关
const onCategoryChange = (categoryId: number) => {
    const category = typeList.value.find(item => item.id === categoryId)
    selectedCategory.value = category
    selectedSubCategory.value = null
    if (currentForm.value) {
        currentForm.value.tid = 0
        // 上一行会触发watch事件，导致selectedCategory被清空
        nextTick(() => {
            selectedCategory.value = category
        })
    }
}

const onSubCategoryChange = (subCategoryId: number) => {
    if (selectedCategory.value && selectedCategory.value.children) {
        const subCategory = selectedCategory.value.children.find(
            (item: any) => item.id === subCategoryId
        )
        selectedSubCategory.value = subCategory
        if (currentForm.value) {
            currentForm.value.tid = subCategoryId
        }
        // 选择子分区后关闭popover
        categoryPopoverVisible.value = false
    }
}

// 根据tid设置选中的分区
const setSelectedCategoryByTid = (tid: number) => {
    for (const category of typeList.value) {
        if (category.children) {
            const subCategory = category.children.find((item: any) => item.id === tid)
            if (subCategory) {
                selectedCategory.value = category
                selectedSubCategory.value = subCategory
                return
            }
        }
    }
}

// 选择封面
const selectCoverWithTauri = async () => {
    try {
        const selected = await open({
            multiple: false,
            defaultPath: getDefaultCoverDir() || undefined,
            filters: [
                {
                    name: 'Image',
                    extensions: ['jpg', 'jpeg', 'png', 'pjp', 'pjpeg', 'jiff', 'gif']
                }
            ]
        })

        if (!selected || selected.length === 0) {
            utilsStore.showMessage('未选择任何封面文件', 'warning')
            return
        }

        if (selectedUser.value && currentTemplate.value && currentForm.value) {
            coverLoading.value = true
            templateLoading.value = true
            const url = await utilsStore.uploadCover(selectedUser.value.uid, selected)
            if (url) {
                currentTemplate.value.cover = url
                currentForm.value.cover = url
            } else {
                throw new Error('封面上传失败')
            }
            const coverPath = Array.isArray(selected) ? selected[0] : selected
            const folder = coverPath.split(/[/\\]/).slice(0, -1).join('\\')
            if (folder) setDefaultCoverDir(folder)
        } else {
            utilsStore.showMessage('请先选择用户和模板', 'error')
        }
    } catch (error) {
        console.error('封面选择失败: ', error)
        utilsStore.showMessage(`'封面选择失败: ${error}'`, 'error')
        return
    } finally {
        coverLoading.value = false
        templateLoading.value = false
    }
}

const clearCurrentCover = () => {
    if (templateLoading.value) {
        return
    }

    if (!currentForm.value || !currentTemplate.value) {
        utilsStore.showMessage('请先选择用户和模板', 'warning')
        return
    }

    currentTemplate.value.cover = ''
    currentForm.value.cover = ''
    coverDisplayUrl.value = ''
    utilsStore.showMessage('已清除封面', 'success')
}

// 使用 Tauri 文件对话框选择视频文件
const selectVideoWithTauri = async () => {
    if (templateLoading.value) {
        utilsStore.showMessage('模板加载中', 'warning')
        return
    }

    templateLoading.value = true
    try {
        const selected = await open({
            multiple: true,
            defaultPath: getDefaultVideoDir() || undefined,
            filters: [
                {
                    name: 'Video',
                    extensions: [
                        'mp4',
                        'flv',
                        'avi',
                        'wmv',
                        'mov',
                        'webm',
                        'mpeg4',
                        'ts',
                        'mpg',
                        'rm',
                        'rmvb',
                        'mkv',
                        'm4v'
                    ]
                }
            ]
        })

        var added = 0

        if (selected && Array.isArray(selected)) {
            const selectedPaths = selected as string[]
            for (const videoPath of selectedPaths) {
                added += await addVideoToCurrentForm(videoPath)
            }

            utilsStore.showMessage(`已选择 ${added} 个文件`, 'success')
            if (selectedPaths.length > 0) {
                const firstPath = selectedPaths[0] ? String(selectedPaths[0]) : ''
                const folder = firstPath
                    ? firstPath.split(/[/\\]/).slice(0, -1).join('\\')
                    : ''
                if (folder) setDefaultVideoDir(folder)
            }
        } else if (typeof selected === 'string') {
            const selectedPath = String(selected)
            added += await addVideoToCurrentForm(selectedPath)
            utilsStore.showMessage(`已选择 ${added} 个文件`, 'success')
            const folder = selectedPath.split(/[/\\]/).slice(0, -1).join('\\')
            if (folder) setDefaultVideoDir(folder)
        }
    } catch (error) {
        console.error('文件选择失败: ', error)
        utilsStore.showMessage(`'文件选择失败: ${error}'`, 'error')
    } finally {
        templateLoading.value = false
    }
}

// 清空所有文件
const clearAllVideos = async () => {
    if (!currentForm.value?.videos || currentForm.value.videos.length === 0) {
        return
    }

    const videoCount = currentForm.value.videos.length
    const videoText = videoCount === 1 ? '1 个文件' : `${videoCount} 个文件`

    templateLoading.value = true
    try {
        await ElMessageBox.confirm(`确定要清空所有已选择的 ${videoText} 吗？`, '确认清空文件', {
            confirmButtonText: '确定清空',
            cancelButtonText: '取消',
            type: 'warning',
            dangerouslyUseHTMLString: false
        })

        // 取消所有对应的上传任务
        const videoIds = currentForm.value.videos.map(video => video.id)
        const correspondingTasks = uploadStore.uploadQueue.filter(task =>
            videoIds.includes(task.video?.id)
        )

        for (const task of correspondingTasks) {
            try {
                await uploadStore.cancelUpload(task.id)
                console.log(`已取消对应的上传任务: ${task.id}`)
            } catch (error) {
                console.error('取消上传任务失败:', error)
                // 继续处理其他任务
            }
        }

        // 清空视频文件列表
        currentForm.value.videos = []
        utilsStore.showMessage(`已清空 ${videoText}`, 'success')
    } catch {
        // 用户取消了操作
    } finally {
        templateLoading.value = false
    }
}

// 调整视频顺序
const removeUploadedFile = async (videoId: string) => {
    if (!currentForm.value?.videos) {
        return
    }

    templateLoading.value = true
    const videoIndex = currentForm.value.videos.findIndex(f => f.id === videoId)
    if (videoIndex > -1) {
        const video = currentForm.value.videos[videoIndex]

        try {
            // 添加确认弹窗
            await ElMessageBox.confirm(
                `确定要删除视频文件"${video.title}"吗？此操作不可撤销。`,
                '确认删除文件',
                {
                    confirmButtonText: '确定删除',
                    cancelButtonText: '取消',
                    type: 'warning'
                }
            )

            // 先查找并取消对应的上传任务
            const correspondingTask = uploadStore.uploadQueue.find(
                task => task.video?.id === videoId
            )
            if (correspondingTask) {
                try {
                    await uploadStore.cancelUpload(correspondingTask.id)
                    console.log(`已取消对应的上传任务: ${correspondingTask.id}`)
                } catch (error) {
                    console.error('取消上传任务失败:', error)
                    // 即使取消失败，仍然继续删除文件
                }
            }

            // 删除视频文件
            currentForm.value.videos.splice(videoIndex, 1)

            utilsStore.showMessage('文件删除成功', 'success')
        } catch (error) {
            // 如果用户取消了确认框，不显示错误消息
            if (error !== 'cancel') {
                console.error('删除文件失败:', error)
                utilsStore.showMessage(`删除文件失败: ${error}`, 'error')
            }
        }
    }
    templateLoading.value = false
}

// 上传相关
const createUpload = async () => {
    // 检查是否有文件可上传
    const hasUploadedFiles = currentForm.value?.videos && currentForm.value.videos.length > 0

    if (!hasUploadedFiles) {
        utilsStore.showMessage('请先选择视频文件', 'error')
        return
    }

    if (!selectedUser.value) {
        utilsStore.showMessage('请先选择用户', 'error')
        return
    }

    uploading.value = true
    try {
        if (currentForm.value) {
            console.log('开始上传文件:', currentForm.value.videos)
            // 确保传递的是正确格式的数组
            const num_added = await uploadStore.createUploadTask(
                selectedUser.value.uid,
                currentTemplateName.value,
                currentForm.value.videos
            )
            utilsStore.showMessage(`添加 ${num_added} 个文件到上传队列`, 'success')
        }

        // 如果启用自动开始，则自动开始任务
        if (userConfigStore.configRoot?.auto_start) {
            setTimeout(async () => {
                try {
                    await autoStartWaitingTasks()
                } catch (error) {
                    console.error('自动开始任务失败:', error)
                }
            }, 500)
        }
    } catch (error) {
        console.error('上传失败: ', error)
        utilsStore.showMessage(`上传失败: ${error}`, 'error')
    } finally {
        uploading.value = false
    }
}

// 处理文件夹监控添加视频事件
const handleAddVideosToForm = async (newVideos: any[]) => {
    templateLoading.value = true
    for (const item of newVideos) {
        try {
            if (typeof item === 'string') {
                await addVideoToCurrentForm(item)
            } else if (item?.path) {
                await addVideoToCurrentForm(item.path, item.title)
            }
        } catch (error) {
            console.error('添加视频失败:', item, error)
        }
    }
    templateLoading.value = false
}

// 处理文件夹监控提交稿件事件
const handleSubmitTemplate = async () => {
    await submitTemplate()
}

// 自动开始待处理的任务
const autoStartWaitingTasks = async () => {
    if (!userConfigStore.configRoot?.auto_start) {
        return
    }

    // 刷新上传队列获取最新状态
    await uploadStore.getUploadQueue()

    // 获取所有待处理的任务
    const pendingTasks = uploadStore.uploadQueue.filter(task => task.status === 'Waiting')

    for (const task of pendingTasks) {
        try {
            await uploadStore.startUpload(task.id)
            console.log(`自动开始任务: ${task.id}`)
        } catch (error) {
            console.error(`自动开始任务失败 ${task.id}:`, error)
            // 继续处理下一个任务
        }
    }
}

// 检查是否所有文件都已上传完成
const allFilesUploaded = computed(() => {
    if (!currentForm.value?.videos || currentForm.value.videos.length === 0) {
        return false
    }
    return currentForm.value.videos.every(video => isVideoUploadCompleted(video))
})

// 提交视频
const submitTemplate = async () => {
    if (!currentTemplateName.value || !selectedUser.value) {
        utilsStore.showMessage('请选择模板', 'error')
        return
    }

    if (!allFilesUploaded.value) {
        const currentAutoSubmitting = getCurrentAutoSubmitting.value
        if (!currentAutoSubmitting) {
            // 首次点击，开始自动提交
            // 将当前video列表加入upload queue
            try {
                if (currentForm.value?.videos && currentForm.value.videos.length > 0) {
                    await uploadStore.createUploadTask(
                        selectedUser.value.uid,
                        currentTemplateName.value,
                        currentForm.value.videos
                    )

                    setTimeout(async () => {
                        try {
                            await autoStartWaitingTasks()
                        } catch (error) {
                            console.error('自动开始任务失败:', error)
                        }
                    }, 500)
                }
            } catch (error) {
                console.error('添加到上传队列失败:', error)
                utilsStore.showMessage(`添加到上传队列失败: ${error}`, 'error')
            }
            if (!activeSessionId.value) {
                utilsStore.showMessage('当前没有可用的编辑标签页', 'error')
                return
            }
            setAutoSubmitting(activeSessionId.value, true)
            startAutoSubmitCheck()
            utilsStore.showMessage('已启动自动提交，上传完成后将自动提交', 'info')
        } else {
            // 第二次点击，取消自动提交
            if (activeSessionId.value) {
                setAutoSubmitting(activeSessionId.value, false)
            }
            utilsStore.showMessage('已取消自动提交', 'info')
        }
        return
    } else {
        performTemplateSubmit(selectedUser.value.uid, currentTemplateName.value, currentForm.value)
    }
}

// 模板名编辑相关函数
const startEditTemplateName = () => {
    isEditingTemplateName.value = true
    editingTemplateName.value = currentTemplateName.value
    nextTick(() => {
        templateNameInputRef.value?.focus()
    })
}

const saveTemplateName = async () => {
    const newName = editingTemplateName.value.trim()

    if (!newName) {
        utilsStore.showMessage('模板名称不能为空', 'error')
        cancelEditTemplateName()
        return
    }

    if (newName === currentTemplateName.value) {
        cancelEditTemplateName()
        return
    }

    if (!selectedUser.value) {
        utilsStore.showMessage('未选择用户', 'error')
        cancelEditTemplateName()
        return
    }

    try {
        // 检查是否已存在同名模板
        const existingTemplate = userConfigStore.getUserTemplate(selectedUser.value.uid, newName)
        if (existingTemplate) {
            utilsStore.showMessage('该名称的模板已存在，请使用其他名称', 'error')
            return
        }

        // 获取原模板配置
        const originalTemplate = userConfigStore.getUserTemplate(
            selectedUser.value.uid,
            currentTemplateName.value
        )
        if (!originalTemplate) {
            utilsStore.showMessage('原模板不存在', 'error')
            cancelEditTemplateName()
            return
        }

        // 先添加新模板
        await userConfigStore.addUserTemplate(selectedUser.value.uid, newName, originalTemplate)

        // 再删除原模板
        await userConfigStore.removeUserTemplate(selectedUser.value.uid, currentTemplateName.value)

        editSessions.value.forEach(session => {
            if (session.uid === selectedUser.value.uid && session.templateName === currentTemplateName.value) {
                session.templateName = newName
                session.tabTitle = session.tabTitle.replace(currentTemplateName.value, newName)
            }
        })
        schedulePersistSessions()

        // 更新当前选择
        currentTemplateName.value = newName

        utilsStore.showMessage('模板重命名成功', 'success')
        isEditingTemplateName.value = false
    } catch (error) {
        console.error('重命名模板失败: ', error)
        utilsStore.showMessage(`重命名模板失败: ${error}`, 'error')
        cancelEditTemplateName()
    }
}

const cancelEditTemplateName = () => {
    isEditingTemplateName.value = false
    editingTemplateName.value = ''
}

// 用户配置相关方法
const openUserConfig = (user: any) => {
    if (user?.expired) {
        showLoginDialog.value = true
        utilsStore.showMessage('该用户 Cookie 已过期，请重新登录', 'warning')
        return
    }

    configUser.value = user
    userConfigVisible.value = true
}

// 检查用户是否有上传任务
const isUserHasUploadTasks = (uid: number) => {
    return uploadStore.uploadQueue.some((task: any) => task.user?.uid === uid)
}

// 处理用户登出
const handleLogoutUser = async (uid: number) => {
    // 如果用户有上传任务，不允许登出
    if (isUserHasUploadTasks(uid)) {
        utilsStore.showMessage('用户有未完成的上传任务，无法登出', 'success')
        return
    }

    try {
        const success = await authStore.logoutUser(uid)
        if (success) {
            const targetSessionIds = editSessions.value
                .filter(session => session.uid === uid)
                .map(session => session.id)
            for (const sessionId of targetSessionIds) {
                await closeSessionTab(sessionId)
            }

            // 如果登出的用户正是当前选择的用户，清除相关记录
            if (selectedUser.value?.uid === uid) {
                selectedUser.value = null
                currentTemplateName.value = ''
                localStorage.removeItem(TEMPLATE_SELECTION_KEY)
            } else {
                // 检查localStorage中记录的用户是否是被登出的用户
                try {
                    const saved = localStorage.getItem(TEMPLATE_SELECTION_KEY)
                    if (saved) {
                        const selection = JSON.parse(saved)
                        if (selection.userUid === uid) {
                            localStorage.removeItem(TEMPLATE_SELECTION_KEY)
                        }
                    }
                } catch (error) {
                    console.error('清理localStorage记录失败:', error)
                }
            }

            utilsStore.showMessage('用户已登出', 'success')
            // 刷新前端数据
            await refreshAllData()
        } else {
            utilsStore.showMessage('登出失败', 'error')
        }
    } catch (error) {
        // 如果用户取消了确认框，error会是'cancel'，不需要显示错误
        if (error !== 'cancel') {
            console.error('登出用户失败:', error)
            utilsStore.showMessage(`登出失败: ${error}`, 'error')
        }
    }
}

// 刷新所有数据的方法
const refreshAllData = async () => {
    try {
        // 重新获取登录用户
        await authStore.getLoginUsers()
        // 重新构建用户模板
        await userConfigStore.buildUserTemplates(authStore.loginUsers)
        // 重新加载用户配置
        await userConfigStore.loadConfig()
        // 重写
        await userConfigStore.saveConfig()
    } catch (error) {
        console.error('刷新数据失败:', error)
    }
}

// 导出日志
const exportLogs = async () => {
    try {
        const now = new Date()
        const localDate = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(
            now.getDate()
        ).padStart(2, '0')}`
        const { value: selectedDate } = await ElMessageBox.prompt(
            '请输入日志日期（YYYY-MM-DD），留空表示全部日志',
            '导出日志',
            {
                confirmButtonText: '下一步',
                cancelButtonText: '取消',
                inputPlaceholder: '例如 2026-03-23',
                inputValue: localDate
            }
        )
        const dateFilter = selectedDate?.trim()
        const zipName = dateFilter
            ? `logs_export_${dateFilter.replace(/-/g, '')}.zip`
            : `logs_export_${localDate.replace(/-/g, '')}.zip`

        const savePath = await save({
            defaultPath: zipName,
            filters: [{ name: 'ZIP', extensions: ['zip'] }]
        })

        if (savePath) {
            await utilsStore.exportLogs(savePath, dateFilter || undefined)
            console.log('日志导出成功：', savePath)
        }
    } catch (error) {
        if (error !== 'cancel') {
            console.error('导出日志失败:', error)
        }
    }
}

// 检查视频转码状态
const checkVideoStatus = async () => {
    if (!selectedUser.value || !currentTemplate.value?.aid) return

    try {
        // 先刷新模板数据
        await ElMessageBox.confirm(
            `此操作会重新拉取模板数据，此操作会丢失未保存的更改，是否继续？`,
            '',
            {
                confirmButtonText: '刷新并继续',
                cancelButtonText: '不刷新，仅显示当前',
                type: 'info'
            }
        )
        await reloadTemplateFromAV(selectedUser.value.uid, currentTemplate.value.aid)
        // 然后显示状态对话框
        showVideoStatusDialog.value = true
    } catch (error) {
        console.error('刷新模板数据失败:', error)
        // 即使刷新失败也显示对话框
        showVideoStatusDialog.value = true
    }
}

// 检查更新
const checkUpdate = async () => {
    try {
        const updateInfo = await utilsStore.checkUpdate()
        if (updateInfo) {
            // 如果有更新，显示确认对话框
            try {
                await ElMessageBox.confirm(`发现新版本 ${updateInfo}，是否前往下载？`, '发现更新', {
                    confirmButtonText: '前往下载',
                    cancelButtonText: '稍后再说',
                    type: 'info'
                })
                // 用户确认后打开下载页面
                await openUrl(`https://github.com/biliup/biliup-app-new/releases/tag/${updateInfo}`)
            } catch {
                // 用户取消，不做任何操作
            }
        } else {
            utilsStore.showMessage('当前已是最新版本', 'success')
        }
    } catch (error) {
        console.error('检查更新失败:', error)
    }
}
</script>

<style scoped>
.main-view {
    height: 100vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.header {
    background: #fff;
    border-bottom: 1px solid #e4e7ed;
    padding: 0 20px;
    position: sticky;
    top: 0;
    z-index: 100;
    flex-shrink: 0;
}

.header-content {
    display: flex;
    justify-content: space-between;
    align-items: center;
    height: 100%;
}

.app-title {
    margin: 0;
    color: #303133;
    display: inline-block;
}

.app-version {
    display: inline-block;
}

.header-center {
    display: flex;
    align-items: center;
    gap: 12px;
}

.header-right {
    display: flex;
    align-items: center;
    gap: 20px;
}

.global-config-btn {
    margin-right: 12px;
}

.main-container {
    flex: 1;
    overflow: hidden;
}

.sidebar {
    background: #f5f7fa;
    border-right: 1px solid #e4e7ed;
    padding: 20px;
    display: flex;
    flex-direction: column;
    overflow: auto;
    width: 320px;
    min-width: 280px;
    max-width: 60vw;
    flex: 0 0 auto;
}

.sidebar-resizer {
    width: 8px;
    cursor: col-resize;
    background: linear-gradient(to right, transparent 0, #e5e7eb 50%, transparent 100%);
    user-select: none;
    flex: 0 0 8px;
}

.sidebar-resizer:hover {
    background: linear-gradient(to right, transparent 0, #cbd5e1 50%, transparent 100%);
}

body.sidebar-resizing {
    cursor: col-resize !important;
    user-select: none;
}

.sidebar-content {
    flex: 1;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 transparent;
}

.sidebar-content::-webkit-scrollbar {
    width: 6px;
}

.sidebar-content::-webkit-scrollbar-track {
    background: transparent;
}

.sidebar-content::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.sidebar-content::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

.sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
}

.sidebar-header h3 {
    margin: 0;
    color: #303133;
}

.header-buttons {
    display: flex;
    gap: 8px;
    align-items: center;
}

.highlight-checkbox {
    margin-right: 8px;
    font-size: 12px;
}

.highlight-checkbox :deep(.el-checkbox__label) {
    font-size: 12px;
    color: #606266;
}

.highlight-checkbox-text {
    line-height: 1.2;
    text-align: center;
}

.user-section {
    margin-bottom: 10px;
}

.user-header {
    display: flex;
    align-items: center;
    padding: 10px;
    background: #fff;
    border-radius: 6px;
    cursor: pointer;
    transition: background-color 0.3s;
}

.user-header:hover {
    background: #ecf5ff;
}

.user-header.user-header-expired {
    background: #f3f4f6;
}

.user-header.user-header-expired:hover {
    background: #f3f4f6;
}

.user-avatar {
    margin-right: 10px;
}

.user-avatar.user-avatar-expired {
    background: #000000;
    color: #ffffff;
}

.user-name {
    flex: 1;
    font-weight: 500;
}

.template-count-badge {
    margin-right: 10px;
}

.template-count-badge :deep(.el-badge__content) {
    background-color: #909399 !important;
    color: #ffffff !important;
    border: none !important;
}

.expand-icon {
    transition: transform 0.3s;
}

.expand-icon.expanded {
    transform: rotate(180deg);
}

.config-icon {
    color: #909399;
    cursor: pointer;
    margin-left: 8px;
    margin-right: 4px;
    transition: color 0.3s;
}

.config-icon:hover {
    color: #409eff;
}

.config-icon.disabled {
    color: #c0c4cc;
    cursor: not-allowed;
}

.config-icon.disabled:hover {
    color: #c0c4cc;
}

.logout-icon {
    color: #f56c6c;
    cursor: pointer;
    margin-left: 8px;
    margin-right: 4px;
    transition: color 0.3s;
}

.logout-icon:hover {
    color: #f89898;
}

.logout-icon.disabled {
    color: #c0c4cc;
    cursor: not-allowed;
}

.logout-icon.disabled:hover {
    color: #c0c4cc;
}

.template-list {
    margin-left: 20px;
    margin-top: 10px;
    position: relative;
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(300px, 1fr));
    gap: 8px;
    align-items: start;
}

.template-tools {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
    grid-column: 1 / -1;
}

.template-sort-select {
    width: 130px;
}

.template-entry {
    min-width: 0;
}

.template-entry.entry-folder {
    grid-column: 1 / -1;
}

.template-folder-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin: 8px 0 4px;
    padding: 0 6px;
    color: #606266;
    font-size: 12px;
    cursor: pointer;
}

.template-folder-info {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
}

.template-folder-cover {
    width: 28px;
    height: 28px;
    border-radius: 4px;
    object-fit: cover;
    border: 1px solid #e5e7eb;
    background: #f5f7fa;
    flex-shrink: 0;
}

.template-folder-title {
    font-weight: 600;
}

.template-folder-actions {
    display: flex;
    align-items: center;
    gap: 6px;
}

.folder-toggle-icon {
    transition: transform 0.2s ease;
    color: #909399;
}

.folder-toggle-icon.collapsed {
    transform: rotate(-90deg);
}

.template-item {
    display: flex;
    align-items: center;
    padding: 8px 12px;
    background: #fff;
    border-radius: 4px;
    margin-bottom: 5px;
    cursor: pointer;
    transition: all 0.3s;
    position: relative;
    min-width: 0;
}

.template-item[draggable='true'] {
    cursor: grab;
}

.template-item-in-folder {
    margin-left: 10px;
}

.template-cover {
    width: 48px;
    height: 48px;
    border-radius: 4px;
    object-fit: cover;
    border: 1px solid #e5e7eb;
    background: #f5f7fa;
    margin-right: 8px;
    flex-shrink: 0;
}

.move-dialog-content {
    display: flex;
    align-items: center;
    gap: 8px;
}

.move-dialog-select {
    flex: 1;
}

.empty-folder {
    margin-top: 8px;
    color: #909399;
    font-size: 12px;
    text-align: center;
}

.template-item:hover {
    background: #f0f9ff;
}

.template-item.active {
    background: #ecf5ff;
    border-left: 3px solid #409eff;
}

.template-item.auto-submitting {
    position: relative;
    overflow: hidden;
    background: linear-gradient(45deg, #e3f2fd, #f3e5f5);
    border: 2px solid #409eff;
    box-shadow: 0 0 20px rgba(64, 158, 255, 0.4);
    animation: pulse-border 1.5s ease-in-out infinite alternate;
}

.template-item.auto-submitting::before {
    content: '';
    position: absolute;
    top: 0;
    left: -100%;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(64, 158, 255, 0.6), transparent);
    animation: shimmer 1.5s infinite;
    z-index: 1;
}

.template-item.auto-submitting::after {
    content: '⚡ 自动上传中...';
    position: absolute;
    top: 2px;
    right: 6px;
    font-size: 10px;
    color: #409eff;
    font-weight: bold;
    animation: blink 1s infinite;
    z-index: 3;
}

.template-item.auto-submitting .template-main {
    position: relative;
    z-index: 2;
}

.template-item.auto-submitting .template-name {
    color: #1976d2;
    font-weight: 600;
    text-shadow: 0 1px 3px rgba(25, 118, 210, 0.2);
}

@keyframes shimmer {
    0% {
        left: -100%;
    }
    100% {
        left: 100%;
    }
}

@keyframes pulse-border {
    0% {
        border-color: #409eff;
        box-shadow: 0 0 20px rgba(64, 158, 255, 0.4);
    }
    100% {
        border-color: #1890ff;
        box-shadow: 0 0 30px rgba(24, 144, 255, 0.6);
    }
}

@keyframes blink {
    0%,
    50% {
        opacity: 1;
    }
    51%,
    100% {
        opacity: 0.3;
    }
}

.template-item.auto-submitting-simple {
    border: 2px solid #409eff !important;
    background: rgba(64, 158, 255, 0.05) !important;
    position: relative;
}

.template-item.auto-submitting-simple::after {
    content: '⚡ 自动上传中...';
    position: absolute;
    top: 2px;
    right: 6px;
    font-size: 10px;
    color: #409eff;
    font-weight: bold;
    z-index: 3;
}

.template-item.template-loading {
    position: relative;
    background: #f5f7fa;
    border: 2px solid #e4e7ed;
    cursor: not-allowed;
    opacity: 0.8;
}

.template-item.template-loading::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: linear-gradient(90deg, transparent, rgba(230, 244, 255, 0.8), transparent);
    animation: loading-shimmer 1.5s infinite;
    z-index: 1;
}

.template-item.template-loading::after {
    content: '🔄 加载中...';
    position: absolute;
    top: 2px;
    right: 6px;
    font-size: 10px;
    color: #909399;
    font-weight: bold;
    z-index: 3;
    animation: loading-spin 1s linear infinite;
}

.template-item.template-loading .template-main {
    position: relative;
    z-index: 2;
}

.template-item.disabled {
    cursor: not-allowed;
    opacity: 0.55;
    pointer-events: none;
}

.expired-mask {
    position: absolute;
    inset: 0;
    background: rgba(17, 24, 39, 0.45);
    backdrop-filter: blur(1px);
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 8;
}

.expired-mask-content {
    text-align: center;
    color: #ffffff;
    padding: 12px;
}

.expired-mask-title {
    font-size: 14px;
    font-weight: 600;
    margin-bottom: 6px;
}

.expired-mask-desc {
    font-size: 12px;
    opacity: 0.9;
    margin-bottom: 10px;
}

@keyframes loading-shimmer {
    0% {
        transform: translateX(-100%);
    }
    100% {
        transform: translateX(100%);
    }
}

@keyframes loading-spin {
    0% {
        opacity: 1;
    }
    50% {
        opacity: 0.5;
    }
    100% {
        opacity: 1;
    }
}

.template-main {
    flex: 1;
    min-width: 0;
}

.template-name {
    font-weight: 500;
    color: #303133;
    display: flex;
    align-items: center;
    gap: 6px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.unsaved-indicator {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: #f56c6c;
    flex-shrink: 0;
    animation: pulse-red 2s infinite;
}

@keyframes pulse-red {
    0% {
        opacity: 1;
        transform: scale(1);
    }
    50% {
        opacity: 0.7;
        transform: scale(1.1);
    }
    100% {
        opacity: 1;
        transform: scale(1);
    }
}

.template-desc {
    font-size: 12px;
    color: #909399;
    margin-top: 2px;
    overflow: hidden;
    white-space: nowrap;
    text-overflow: ellipsis;
}

.main-content {
    padding: 0;
    overflow: hidden;
}

.content-wrapper {
    height: 100%;
    padding: 20px;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 transparent;
}

.content-wrapper::-webkit-scrollbar {
    width: 6px;
}

.content-wrapper::-webkit-scrollbar-track {
    background: transparent;
}

.content-wrapper::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.content-wrapper::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

.no-selection,
.no-template {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

.form-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 20px;
    padding-bottom: 15px;
    border-bottom: 1px solid #e4e7ed;
}

.form-header h3 {
    margin: 0;
    color: #303133;
}

.editor-tabs {
    margin-bottom: 10px;
}

.editor-tab-label {
    display: inline-block;
    max-width: 220px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.editor-tab-label.auto-submit-tab {
    color: #1d4ed8;
    font-weight: 700;
}

.template-name-container {
    flex: 1;
    margin-right: 20px;
}

.edit-bv-template-disaplay {
    display: inline-block;
}

.template-name-display {
    margin: 0;
    color: #303133;
    cursor: pointer;
    padding: 8px 12px;
    border-radius: 4px;
    transition: all 0.3s;
    display: inline-flex;
    align-items: center;
    gap: 8px;
    position: relative;
    max-width: fit-content;
}

.template-name-display:hover {
    background: #f0f9ff;
    color: #409eff;
}

.template-name-display .edit-hint-icon {
    opacity: 0;
    font-size: 14px;
    transition: opacity 0.3s;
}

.template-name-display:hover .edit-hint-icon {
    opacity: 1;
}

.template-name-input {
    max-width: 300px;
}

.refresh-btn {
    cursor: pointer;
    color: #606266;
    font-size: 16px;
    transition: all 0.3s;
    border-radius: 4px;
}

.refresh-btn:hover {
    color: #409eff;
    background-color: #f0f9ff;
    transform: rotate(180deg);
}

.header-actions {
    display: flex;
    gap: 10px;
}

.form-section {
    margin-bottom: 20px;
}

.form-section.drag-target {
    border: 2px dashed #409eff;
    background: rgba(64, 158, 255, 0.05);
    transition: all 0.3s ease;
}

.form-section.drag-target .el-card__header {
    background: rgba(64, 158, 255, 0.1);
}

/* 卡片折叠样式 */
.card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    cursor: pointer;
    user-select: none;
    transition: all 0.3s ease;
    height: 10px;
}

.card-header:hover {
    color: #409eff;
}

.card-header .header-actions {
    display: flex;
    gap: 8px;
    align-items: center;
}

.card-header .header-actions .el-button {
    margin: 0;
    padding: 4px;
    border: none;
    background: transparent;
    transition: all 0.3s ease;
}

.card-header .header-actions .el-button:hover {
    background: rgba(245, 108, 108, 0.1);
    color: #f56c6c;
    transform: scale(1.1);
}

.card-header .header-actions .el-button .el-icon {
    font-size: 14px;
}

.collapse-icon {
    transition: transform 0.3s ease;
    color: #909399;
}

.collapse-icon:hover {
    color: #409eff;
}

.collapse-icon.collapsed {
    transform: rotate(-90deg);
}

.form-section.collapsed {
    margin-bottom: 10px;
}

.card-content {
    padding-top: 0;
}

.drag-hint {
    float: right;
    color: #409eff;
    font-size: 12px;
    font-weight: 500;
}

.cover-uploader-row {
    display: inline-flex;
    align-items: center;
    gap: 10px;
}

.cover-uploader {
    position: relative;
    display: inline-block;
    z-index: 1; /* 确保容器有基础层级 */
}

.cover-uploader .cover-image {
    width: 100px;
    height: 60px;
    object-fit: cover;
    border-radius: 4px;
    transition:
        transform 0.3s ease,
        box-shadow 0.3s ease;
    cursor: pointer;
    position: relative; /* 重要：让 z-index 生效 */
}

.cover-clear-btn-side {
    align-self: center;
    background: transparent;
    border: none;
    box-shadow: none;
    color: #9ca3af;
    padding: 0;
    min-width: 14px;
    width: 14px;
    height: 14px;
    line-height: 14px;
    font-size: 12px;
    transition: opacity 0.2s ease;
}

.cover-clear-btn-side:hover {
    background: transparent;
    border: none;
    color: #ef4444;
}

.cover-clear-btn-side :deep(.el-icon) {
    font-size: 12px;
}

.cover-uploader:hover + .cover-clear-btn-side {
    opacity: 0;
    pointer-events: none;
}

.cover-uploader .cover-image:hover {
    transform: scale(3) translateX(25px);
    box-shadow: 0 8px 16px rgba(0, 0, 0, 0.2);
    z-index: 999; /* 确保悬浮时在最顶层 */
    position: relative; /* 确保定位生效 */
}

.cover-uploader-icon {
    width: 100px;
    height: 60px;
    border: 1px dashed #d9d9d9;
    border-radius: 4px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #8c939d;
    font-size: 24px;
}

.upload-tip {
    color: #909399;
    font-size: 12px;
    margin-top: 5px;
}

.video-buttons-group {
    display: flex;
    gap: 10px;
    align-items: center;
    margin-bottom: 10px;
}

.drag-active-tip {
    color: #409eff !important;
    font-weight: 500;
    animation: pulse 1.5s infinite;
}

@keyframes pulse {
    0% {
        opacity: 1;
    }
    50% {
        opacity: 0.7;
    }
    100% {
        opacity: 1;
    }
}

.empty-users {
    text-align: center;
    margin-top: 50px;
}

/* 登录对话框样式 */
.login-dialog :deep(.el-dialog) {
    margin: 0;
    padding: 0;
    border-radius: 0;
    background: transparent;
    box-shadow: none;
    max-height: 90vh;
}

.login-dialog :deep(.el-dialog__header) {
    display: none;
}

.login-dialog :deep(.el-dialog__body) {
    padding: 0;
    max-height: 90vh;
    overflow: hidden;
}

.login-dialog-content {
    max-height: 90vh;
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: #c1c1c1 transparent;
}

.login-dialog-content::-webkit-scrollbar {
    width: 6px;
}

.login-dialog-content::-webkit-scrollbar-track {
    background: transparent;
}

.login-dialog-content::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.login-dialog-content::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

.login-dialog-content .login-view {
    min-height: auto;
    padding: 0;
    background: transparent;
}

.login-dialog-content .login-container {
    max-width: none;
    width: 100%;
    padding: 0;
}

.login-dialog-content .login-card {
    margin: 0;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.2);
    border-radius: 16px;
}

.checkbox-group {
    display: flex;
    flex-direction: column;
    gap: 12px;
}

.interactive-setting-row {
    display: inline-flex;
    align-items: center;
    gap: 8px;
}

.interactive-help-icon {
    color: #909399;
    cursor: pointer;
    font-size: 14px;
    transition: color 0.2s ease;
}

.interactive-help-icon:hover {
    color: #606266;
}

/* 表单提示样式 */
.form-tip {
    font-size: 12px;
    color: #909399;
    margin-top: 5px;
    line-height: 1.4;
}

.form-tip div {
    margin-bottom: 2px;
}

/* 分区选择器样式 */
.category-trigger {
    width: 100%;
    display: flex !important;
    justify-content: space-between !important;
    align-items: center !important;
    border: 1px solid #dcdfe6;
    background: #fff;
    color: #606266;
    padding: 8px 15px;
    border-radius: 4px;
    cursor: pointer;
    transition: border-color 0.3s;
    position: relative;
}

.category-trigger .category-text {
    flex: 1;
    text-align: left;
    padding-right: 30px; /* 为右侧箭头留出空间 */
}

.category-trigger:hover {
    border-color: #409eff;
}

.category-trigger.el-button--primary {
    background: #fff;
    border-color: #409eff;
    color: #409eff;
}

.category-trigger .placeholder {
    color: #c0c4cc;
}

.category-trigger .arrow-icon {
    position: absolute;
    right: 12px;
    top: 50%;
    transform: translateY(-50%);
    transition: transform 0.3s;
    flex-shrink: 0;
}

/* 分区选择面板 */
.category-selector-panel {
    display: flex;
    height: 360px;
    border-radius: 6px;
    overflow: hidden;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.category-list {
    width: 180px;
    background: #f8f9fa;
    border-right: 1px solid #e9ecef;
    overflow-y: auto;
}

.subcategory-list {
    flex: 1;
    background: #fff;
    overflow-y: auto;
}

.category-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    cursor: pointer;
    transition: all 0.3s;
    border-bottom: 1px solid #f0f2f5;
}

.category-item:hover {
    background: #e6f7ff;
    color: #1890ff;
}

.category-item.active {
    background: #1890ff;
    color: #fff;
}

.category-item.active .arrow-right {
    color: #fff;
}

.category-name {
    font-size: 13px;
}

.arrow-right {
    color: #c0c4cc;
    font-size: 12px;
    transition: color 0.3s;
}

.subcategory-item {
    padding: 12px 16px;
    cursor: pointer;
    transition: all 0.3s;
    border-bottom: 1px solid #f0f2f5;
}

.subcategory-item:hover {
    background: #f0f9ff;
}

.subcategory-item.active {
    background: #e6f7ff;
    border-left: 3px solid #1890ff;
}

.subcategory-content {
    display: flex;
    flex-direction: column;
    gap: 4px;
}

.subcategory-name {
    font-size: 14px;
    font-weight: 500;
    color: #303133;
}

.subcategory-desc {
    font-size: 12px;
    color: #909399;
    line-height: 1.4;
}

.empty-subcategory {
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
}

/* 滚动条样式 */
.category-list::-webkit-scrollbar,
.subcategory-list::-webkit-scrollbar {
    width: 6px;
}

.category-list::-webkit-scrollbar-track,
.subcategory-list::-webkit-scrollbar-track {
    background: transparent;
}

.category-list::-webkit-scrollbar-thumb,
.subcategory-list::-webkit-scrollbar-thumb {
    background-color: #c1c1c1;
    border-radius: 3px;
}

.category-list::-webkit-scrollbar-thumb:hover,
.subcategory-list::-webkit-scrollbar-thumb:hover {
    background-color: #a8a8a8;
}

/* 上传操作区域 */
.upload-actions {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 16px;
    padding: 20px 0;
    margin-top: 20px;
    border-top: 1px solid #e4e7ed;
}

.upload-actions .el-button {
    min-width: 140px;
}

/* 拖拽覆盖层样式 */
.drag-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: rgba(64, 158, 255, 0.9);
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(4px);
}

.drag-content {
    text-align: center;
    color: white;
    padding: 40px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.1);
    border: 2px dashed rgba(255, 255, 255, 0.8);
    max-width: 500px;
}

.drag-icon {
    font-size: 64px;
    margin-bottom: 20px;
    animation: bounce 2s infinite;
}

@keyframes bounce {
    0%,
    20%,
    50%,
    80%,
    100% {
        transform: translateY(0);
    }
    40% {
        transform: translateY(-10px);
    }
    60% {
        transform: translateY(-5px);
    }
}

.drag-content h3 {
    margin: 0 0 10px 0;
    font-size: 24px;
    font-weight: 600;
}

.drag-content p {
    margin: 8px 0;
    font-size: 16px;
    opacity: 0.9;
}

.drag-content .warning-text {
    color: #ffd700;
    font-weight: 500;
    margin-top: 15px;
}

/* 禁用状态样式 */
.cover-uploader.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
}

.cover-uploader.disabled:hover {
    border-color: #dcdfe6 !important;
}

.template-name-display.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
    color: #909399 !important;
}

.template-name-display.disabled .edit-hint-icon {
    color: #c0c4cc !important;
}

/* 用户头部禁用状态 */
.user-header.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
}

.user-header.disabled:hover {
    background: #fff !important;
}

/* 模板项禁用状态 */
.template-item.disabled {
    cursor: not-allowed !important;
    opacity: 0.6 !important;
}

.template-item.disabled:hover {
    background: #fff !important;
}
</style>

<style>
/* 全局样式：分区选择器popover */
.category-popover {
    padding: 0 !important;
}

.category-popover .el-popover__arrow {
    display: none;
}

.interactive-confirm-dialog {
    display: flex;
    flex-direction: column;
    gap: 10px;
}

.interactive-confirm-dialog-text {
    line-height: 1.6;
}

.interactive-confirm-dialog-checkbox {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    user-select: none;
}

.interactive-confirm-dialog-checkbox input {
    cursor: pointer;
}
</style>
