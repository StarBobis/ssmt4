<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import { gamesList, appSettings } from '../store';
import { open } from '@tauri-apps/plugin-dialog';
import { Folder, Refresh, Picture, Search, Plus, Edit, Delete, FolderAdd, ArrowRight, Sort } from '@element-plus/icons-vue';
import { ElMessage, ElMessageBox } from 'element-plus';

interface ModInfo {
    id: string;
    name: string;
    enabled: boolean;
    path: string;
    relativePath: string;
    previewImages: string[];
    group: string;
    isDir: boolean;
    last_modified: number;
}

interface ArchivePreview {
    root_dirs: string[];
    file_count: number;
    has_ini: boolean;
    format: string;
}

// Context Menu State
const contextMenu = reactive({
    visible: false,
    x: 0,
    y: 0,
    type: 'mod' as 'mod' | 'group',
    target: null as any
});

const closeContextMenu = () => {
    contextMenu.visible = false;
};

// Group Management
const createNewGroup = async () => {
    try {
        const result = await ElMessageBox.prompt('请输入新分类名称', '新建分类', {
            confirmButtonText: '创建',
            cancelButtonText: '取消',
        }) as any;
        
        const value = result.value;
        
        if (value) {
            await invoke('create_mod_group', { 
                gameName: selectedGame.value,
                groupName: value 
            });
            ElMessage.success('分类创建成功');
            // Refresh logic usually handled by watcher, but manual refresh is safer
             fetchMods();
        }
    } catch {
        // User cancelled
    }
};

const renameGroup = async (oldName: string) => {
    try {
        const result = await ElMessageBox.prompt('请输入新的分类名称', '重命名分类', {
            confirmButtonText: '确定',
            cancelButtonText: '取消',
            inputValue: oldName
        }) as any;
        
        const value = result.value;
        
        if (value && value !== oldName) {
             await invoke('rename_mod_group', { 
                gameName: selectedGame.value, 
                oldGroup: oldName,
                newGroup: value 
            });
            ElMessage.success('分类重命名成功');
            if (selectedGroup.value === oldName) {
                selectedGroup.value = value;
            }
            fetchMods();
        }
    } catch {
        // User cancelled
    }
};

const deleteGroup = async (groupName: string) => {
    try {
        await ElMessageBox.confirm(
            `确定要删除分类 "${groupName}" 吗？这会将文件夹移动到回收站。`,
            '删除分类',
            {
                confirmButtonText: '删除',
                cancelButtonText: '取消',
                type: 'warning',
            }
        )
        
        await invoke('delete_mod_group', {
            gameName: selectedGame.value,
            groupName: groupName
        });
        
        ElMessage.success('分类已删除');
        if (selectedGroup.value === groupName) {
            selectedGroup.value = 'All';
        }
        fetchMods();
    } catch (e: any) {
        if (e !== 'cancel') {
             ElMessage.error(`删除失败: ${e}`);
        }
    }
};

const moveModToGroup = async (mod: ModInfo, groupName: string) => {
    try {
        await invoke('move_mod_to_group', {
            gameName: selectedGame.value,
            modId: mod.id, // relative path
            newGroup: groupName
        });
        ElMessage.success(`移动到 ${groupName || 'Root'} 成功`);
        // fetchMods handled by watcher mostly
    } catch (e: any) {
        ElMessage.error('移动失败: ' + e);
    }
};

const showGroupContextMenu = (e: MouseEvent, group: string) => {
    if (group === 'All' || group === 'Root') return;
    contextMenu.visible = true;
    contextMenu.x = e.clientX;
    contextMenu.y = e.clientY;
    contextMenu.type = 'group';
    contextMenu.target = group;
};

const showModContextMenu = (e: MouseEvent, mod: ModInfo) => {
    contextMenu.visible = true;
    contextMenu.x = e.clientX;
    contextMenu.y = e.clientY;
    contextMenu.type = 'mod';
    contextMenu.target = mod;
};


interface GroupInfo {
    id: string; // Full path
    name: string; // Display name
    iconPath?: string;
}

const loading = ref(false);
const mods = ref<ModInfo[]>([]);
const availableGroups = ref<GroupInfo[]>([]);
const selectedGame = ref('');
const searchQuery = ref('');
const selectedGroup = ref('All');
// Sorting state
const sortBy = ref<'name' | 'date' | 'status'>('date');
const sortOrder = ref<'asc' | 'desc'>('desc');

// Install Dialog State
const showInstallDialog = ref(false);
const installForm = reactive({
    archivePath: '',
    modName: '',
    targetGroup: '',
    password: ''
});
const installPreview = ref<ArchivePreview | null>(null);
const isInstalling = ref(false);

// Watcher cleanup
let unlistenFileChange: UnlistenFn | null = null;
let unlistenDrop: UnlistenFn | null = null;
let debounceTimer: ReturnType<typeof setTimeout> | null = null;

// Initialize selected game from store if possible
onMounted(async () => {
    // ... existing init code ...
    if (appSettings.currentConfigName && gamesList.find(g => g.name === appSettings.currentConfigName)) {
        selectedGame.value = appSettings.currentConfigName;
    } else if (gamesList.length > 0) {
        selectedGame.value = gamesList[0].name;
    }
    
    // Listen for file drops
    unlistenDrop = await listen('tauri://drag-drop', async (event: any) => {
        const payload = event.payload;
        if (payload.paths && payload.paths.length > 0) {
            handleFileDrop(payload.paths[0]);
        }
    });

    // Start listening for file changes
    unlistenFileChange = await listen('mod-filesystem-changed', () => {
        // Debounce the refresh
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
            console.log("File system changed, refreshing...");
            // Silent refresh (no loading spinner to avoid flickering)
            silentRefresh();
        }, 500); // 500ms debounce
    });
    
    if (selectedGame.value) {
        // Initial load
        startWatching(selectedGame.value);
    }
});

onUnmounted(() => {
    if (unlistenFileChange) unlistenFileChange();
    if (unlistenDrop) unlistenDrop();
    // Stop watching backend? 
    // Ideally yes, but changing pages shouldn't necessarily stop watching if we want background updates. 
    // But for performance, let's stop it.
    invoke('unwatch_mods').catch(e => console.error(e));
});

watch(selectedGame, (newVal) => {
    if (newVal) {
        startWatching(newVal);
        selectedGroup.value = 'All';
    }
});

watch(() => appSettings.currentConfigName, (newVal) => {
    if (newVal && gamesList.find(g => g.name === newVal)) {
        selectedGame.value = newVal;
    }
}, { immediate: true });

const handleFileDrop = async (path: string) => {
    // Check extension
    const lower = path.toLowerCase();
    if (lower.endsWith('.zip') || lower.endsWith('.7z')) {
        installForm.archivePath = path;
        
        // Guess initial name from filename
        const filename = path.split(/[\\/]/).pop() || 'New Mod';
        installForm.modName = filename.replace(/\.(zip|7z|rar)/i, '');
        
        // Default group: if 'Root' or 'All' is selected, default to 'Default'
        // If a specific group is selected, use that.
        installForm.targetGroup = (selectedGroup.value === 'All' || selectedGroup.value === 'Root') ? 'Default' : selectedGroup.value;
        installForm.password = '';
        
        // Load Preview
        try {
            loading.value = true;
            installPreview.value = await invoke('preview_mod_archive', { path });
            showInstallDialog.value = true;
        } catch (e: any) {
            ElMessage.error({
                message: `无法读取压缩包: ${e}`,
                offset: 48
            });
        } finally {
            loading.value = false;
        }
    } else if (lower.endsWith('.rar')) {
         installForm.archivePath = path;
        const filename = path.split(/[\\/]/).pop() || 'New Mod';
        installForm.modName = filename.replace(/\.(zip|7z|rar)/i, '');
        installForm.targetGroup = (selectedGroup.value === 'All' || selectedGroup.value === 'Root') ? 'Default' : selectedGroup.value;
        installForm.password = '';

         try {
            loading.value = true;
            installPreview.value = await invoke('preview_mod_archive', { path });
            showInstallDialog.value = true;
        } catch (e: any) {
            // Show raw error if it's "not supported" to include details
            // Add offset to avoid titlebar
            ElMessage.error({
                message: `RAR 读取失败: ${e}`,
                offset: 48
            });
        } finally {
            loading.value = false;
        }
    }
};

const confirmInstall = async () => {
    if (!installForm.modName) {
        ElMessage.warning({ message: '请输入 Mod 名称', offset: 48 });
        return;
    }
    
    isInstalling.value = true;
    try {
        await invoke('install_mod_archive', {
            gameName: selectedGame.value,
            archivePath: installForm.archivePath,
            targetName: installForm.modName,
            targetGroup: installForm.targetGroup,
            password: installForm.password || null
        });
        ElMessage.success({ message: '安装成功！', offset: 48 });
        showInstallDialog.value = false;
        // Refresh handled by watcher
    } catch (e) {
        ElMessage.error({ message: `安装失败: ${e}`, offset: 48 });
    } finally {
        isInstalling.value = false;
    }
};


// ... existing code ...

const startWatching = async (gameName: string) => {
    loading.value = true;
    try {
        // First load data
        await refreshMods(gameName);
        // Then start watching (which might fail if folder doesn't exist, but that's ok)
        await invoke('watch_mods', { gameName });
    } catch (error) {
        console.error('Failed to start watching:', error);
    } finally {
        loading.value = false;
    }
};

const silentRefresh = async () => {
    if (!selectedGame.value) return;
    try {
        const result = await invoke('scan_mods', { gameName: selectedGame.value }) as { mods: ModInfo[], groups: GroupInfo[] };
        mods.value = result.mods;
        availableGroups.value = result.groups;
    } catch (e) {
        console.error("Silent refresh failed", e);
    }
}

const refreshMods = async (gameName: string) => {
    try {
        const result = await invoke('scan_mods', { gameName }) as { mods: ModInfo[], groups: GroupInfo[] };
        mods.value = result.mods;
        availableGroups.value = result.groups;
    } catch (error) {
        console.error('Failed to scan mods:', error);
        ElMessage.error(`扫描失败: ${error}`);
        mods.value = [];
        availableGroups.value = [];
    }
};

// Removed old fetchMods, replaced by refreshMods/startWatching logics
const fetchMods = () => {
    if (selectedGame.value) startWatching(selectedGame.value);
};

const toggleMod = async (mod: ModInfo) => {
    // Optimistic UI update is risky here if renaming fails, but let's try
    // Better to wait for server response
    const originalState = mod.enabled;
    const targetState = !originalState; // We want to toggle
    
    try {
        await invoke('toggle_mod', { 
            gameName: selectedGame.value, 
            modRelativePath: mod.relativePath,
            enable: targetState
        });
        
        // Refresh list to get new paths
        await silentRefresh();
        ElMessage.success(`${mod.name} ${targetState ? '已启用' : '已禁用'}`);
    } catch (error) {
        console.error('Failed to toggle mod:', error);
        // creating the mod object implies it exists in memory, we might need a revert if we did optimistic
        ElMessage.error(`操作失败: ${error}`);
    }
};

const openModFolder = async (path: string) => {
    try {
        await invoke('open_in_explorer', { path });
    } catch (error) {
        console.error(error);
    }
};

const openGameFolder = async () => {
    try {
        await invoke('open_game_mods_folder', { gameName: selectedGame.value });
    } catch (error) {
        console.error(error);
    }
}

// Computed Properties
const groups = computed(() => {
    // Map of groupID -> GroupInfo
    const map = new Map<string, GroupInfo>();
    
    // Add known groups from backend
    availableGroups.value.forEach(g => {
        map.set(g.id, g);
    });

    // Add implicit groups from mods
    mods.value.forEach(m => {
        if (m.group && m.group !== "Root" && !map.has(m.group)) {
            // Split slash name if we want friendly name for implicit groups
            // ModInfo.group is the full path ID now
            const parts = m.group.split('/');
            const name = parts[parts.length - 1];
            map.set(m.group, { id: m.group, name: name });
        }
    });

    // Sort by ID is usually fine for hierarchy
    const list = Array.from(map.values()).sort((a, b) => a.id.localeCompare(b.id));

    return [{ id: 'All', name: '全部' }, ...list];
});

const groupTree = computed(() => {
    const tree: any[] = [];
    const nodeMap = new Map<string, any>();
    
    // Sort by depth so parents are processed before children
    const sorted = [...(groups.value || [])]
        .filter(g => g.id !== 'All' && g.id !== 'Root')
        .sort((a, b) => a.id.split('/').length - b.id.split('/').length);

    sorted.forEach(g => {
        const parts = g.id.split('/');
        const name = parts[parts.length - 1]; // Use last part as label
        
        const node = {
            id: g.id,
            label: name, // Just the folder name, not full path
            children: [],
            icon: g.iconPath,
            // Count includes mods directly in this group
            // If we want recursive count, we can do post-order traversal later
            count: mods.value.filter(m => m.group === g.id).length
        };
        
        nodeMap.set(g.id, node);

        if (parts.length === 1) {
            tree.push(node);
        } else {
            const parentId = parts.slice(0, -1).join('/');
            const parent = nodeMap.get(parentId);
            if (parent) {
                parent.children.push(node);
            } else {
                // Should not happen if sorted by depth and parents exist
                // Fallback: add to root
                tree.push(node);
            }
        }
    });
    
    return tree;
});

const setGroupIcon = async (groupPath: string) => {
     try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'Image',
                extensions: ['png', 'jpg', 'jpeg', 'bmp', 'webp']
            }]
        });

        if (selected) {
            await invoke('set_mod_group_icon', {
                gameName: selectedGame.value,
                groupPath: groupPath,
                iconPath: selected
            });
            ElMessage.success('图标设置成功');
            // Little hack to refresh image cache? 
            // Usually fetchMods -> rescans -> returns new icon list. 
            // Browser might cache image. convertFileSrc usually handles it? 
            // Sometimes need timestamp query.
            await fetchMods();
        }
    } catch (e: any) {
        ElMessage.error('设置图标失败: ' + e);
    }
};

const openModGroupFolder = async (groupPath: string) => {
    try {
        await invoke('open_mod_group_folder', {
            gameName: selectedGame.value,
            groupPath: groupPath
        });
    } catch (e: any) {
        ElMessage.error('无法打开文件夹: ' + e);
    }
};

const filteredMods = computed(() => {
    let result = mods.value;

    if (selectedGroup.value !== 'All') {
        result = result.filter(m => m.group === selectedGroup.value);
    }

    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        result = result.filter(m => m.name.toLowerCase().includes(query));
    }

    return result.sort((a, b) => {
        let cmp = 0;
        switch (sortBy.value) {
            case 'name':
                cmp = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
                break;
            case 'date':
                cmp = (a.last_modified || 0) - (b.last_modified || 0);
                if (cmp === 0) {
                     // Secondary sort by name if date is same
                     cmp = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
                }
                break;
            case 'status':
                // Enabled first (true > false)
                // Strict: Enabled always > Disabled regarding value
                const valA = a.enabled ? 1 : 0;
                const valB = b.enabled ? 1 : 0;
                cmp = valA - valB;
                 
                // If sorting by status, we usually want ENABLED at Top.
                // If sortOrder is 'desc' (default), 1 - 0 = positive -> 1 is "larger" -> if desc, larger comes first.
                // So (1, 0) -> [1, 0]. Correct (Enabled Top).
                // If asc, [0, 1] -> Disabled Top.
                // User wants "Disabled at bottom", which implies Enabled at Top.
                // If sort order is DESC, then it works.
                // But what if same status? Sort by name secondary
                if (cmp === 0) {
                    // Secondary deterministic name
                     let nameCmp = a.name.localeCompare(b.name, undefined, { numeric: true, sensitivity: 'base' });
                     // Name sort should usually be ASC even if main sort is DESC?
                     // If we return nameCmp here directly, it will be flipped by the final return
                     // so we must account for sortOrder being flipped later
                     if (sortOrder.value === 'desc') nameCmp = -nameCmp; // negate to counteract final flip
                     cmp = nameCmp;
                }
                break;
        }
        return sortOrder.value === 'asc' ? cmp : -cmp;
    });
});

const getPreviewUrl = (mod: ModInfo) => {
    if (mod.previewImages && mod.previewImages.length > 0) {
        return convertFileSrc(mod.previewImages[0]);
    }
    return ''; // Placeholder handled by UI
};

const getGroupIcon = (groupId: string) => {
    if(!groupId || groupId === 'Root') return null;
    const group = availableGroups.value.find(g => g.id === groupId);
    // Loop through implicit groups if not found? 
    // availableGroups usually contains all groups found by scanner.
    return group?.iconPath;
};

</script>

<template>
  <div class="page-container mod-manager">
    <!-- Header Toolbar -->
    <div class="toolbar glass-panel">
        <div class="left-tools">
            <el-input
                v-model="searchQuery"
                placeholder="搜索 Mod..."
                :prefix-icon="Search"
                style="width: 240px"
                clearable
            />
        </div>

        <div class="right-tools">
            <el-dropdown trigger="click" @command="(cmd: any) => { 
                if(cmd.startsWith('order:')) sortOrder = cmd.split(':')[1];
                else sortBy = cmd;
            }">
                <el-button :icon="Sort" plain>
                    排序: {{ sortBy === 'name' ? '名称' : (sortBy === 'date' ? '日期' : '状态') }}
                </el-button>
                <template #dropdown>
                    <el-dropdown-menu>
                        <el-dropdown-item command="date">按修改日期</el-dropdown-item>
                        <el-dropdown-item command="name">按名称</el-dropdown-item>
                        <el-dropdown-item command="status">按状态</el-dropdown-item>
                        <el-dropdown-item divided command="order:asc">升序 (Oldest/A-Z)</el-dropdown-item>
                        <el-dropdown-item command="order:desc">降序 (Newest/Z-A)</el-dropdown-item>
                    </el-dropdown-menu>
                </template>
            </el-dropdown>
            <div class="divider-vertical"></div>
            <el-button @click="openGameFolder" :icon="Folder" plain>文件夹</el-button>
            <el-button @click="fetchMods" :icon="Refresh" :loading="loading" circle type="primary" plain></el-button>
        </div>
    </div>

    <div class="main-content" @click="closeContextMenu" @contextmenu="closeContextMenu">
        <!-- Sidebar Filter -->
        <div class="sidebar glass-panel">
            <div class="sidebar-header">
                <span class="title">分类列表</span>
                <el-button :icon="Plus" circle size="small" @click.stop="createNewGroup" text bg />
            </div>
            <div class="group-list">
                <div 
                    class="group-item" 
                    :class="{ active: selectedGroup === 'All' }"
                    @click="selectedGroup = 'All'"
                >
                    <el-icon class="tree-icon-placeholder"><Folder /></el-icon>
                    <span>全部</span>
                    <span class="count">{{ mods.length }}</span>
                </div>
                <!-- Special Groups for Folder Structure -->
                 <div 
                    class="group-item" 
                    :class="{ active: selectedGroup === 'Root' }"
                     @click="selectedGroup = 'Root'"
                     v-if="mods.some(m => m.group === 'Root')"
                >
                    <el-icon class="tree-icon-placeholder"><Folder /></el-icon>
                    <span>未分类 (Root)</span>
                    <span class="count">{{ mods.filter(m => m.group === 'Root').length }}</span>
                </div>
                
                <el-tree
                    v-if="groupTree.length > 0"
                    :data="groupTree"
                    node-key="id"
                    :props="{ label: 'label', children: 'children' }"
                    :expand-on-click-node="false"
                    :current-node-key="selectedGroup"
                    highlight-current
                    @node-click="(data) => selectedGroup = data.id"
                    class="group-tree"
                >
                    <template #default="{ node, data }">
                        <div class="custom-tree-node"
                             @contextmenu.prevent.stop="showGroupContextMenu($event, data.id)"
                        >
                            <div class="node-content">
                                <img v-if="data.icon" :src="convertFileSrc(data.icon)" class="tree-icon" />
                                <el-icon v-else class="tree-icon-placeholder"><Folder /></el-icon>
                                <span class="node-label" :title="node.label">{{ node.label }}</span>
                            </div>
                            <span class="count" v-if="data.count > 0">{{ data.count }}</span>
                        </div>
                    </template>
                </el-tree>
            </div>
        </div>

        <!-- Mod Grid -->
        <div class="mod-grid-container" v-loading="loading">
            <div v-if="filteredMods.length === 0" class="empty-state">
                <el-empty :description="searchQuery ? '没有找到匹配的 Mod' : '这个游戏还没有 Mod，拖拽压缩包到这里安装！'" >
                    <el-button type="primary" plain @click="openGameFolder">打开文件夹</el-button>
                </el-empty>
            </div>
            
            <div v-else class="mod-grid">
                <div 
                    v-for="mod in filteredMods" 
                    :key="mod.id" 
                    class="mod-card glass-panel"
                    :class="{ 'is-disabled': !mod.enabled }"
                    @contextmenu.prevent.stop="showModContextMenu($event, mod)"
                >
                    <!-- Preview Image -->
                    <div class="card-preview">
                        <div v-if="getPreviewUrl(mod)" class="image-wrapper">
                             <el-image 
                                :src="getPreviewUrl(mod)" 
                                fit="cover" 
                                loading="lazy"
                                style="width: 100%; height: 100%;"
                             >
                                <template #error>
                                    <div class="image-placeholder"><el-icon><Picture /></el-icon></div>
                                </template>
                             </el-image>
                        </div>
                        <div v-else class="image-placeholder">
                            <span class="char-avatar">{{ mod.group === 'Root' ? mod.name.charAt(0) : mod.group.charAt(0) }}</span>
                        </div>

                        <!-- Hover Actions -->
                        <div class="card-overlay">
                            <el-button size="small" circle @click.stop="openModFolder(mod.path)" :icon="Folder" title="打开文件夹" />
                        </div>
                    </div>

                    <!-- Info -->
                    <div class="card-info">
                        <div class="header-row">
                            <div class="text-content">
                                <div class="mod-name" :title="mod.name">{{ mod.name }}</div>
                                <div class="mod-group">
                                    <template v-if="mod.group !== 'Root'">
                                        <img v-if="getGroupIcon(mod.group)" :src="convertFileSrc(getGroupIcon(mod.group)!)" class="mini-group-icon" />
                                        <span>{{ mod.group.split('/').pop() }}</span>
                                    </template>
                                    <span v-else>未分类</span>
                                </div>
                            </div>
                            <el-switch 
                                :model-value="mod.enabled"
                                @change="toggleMod(mod)"
                                inline-prompt
                                active-text="ON"
                                inactive-text="OFF"
                                style="--el-switch-on-color: #13ce66; --el-switch-off-color: #ff4949;"
                            />
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <!-- Install Dialog -->
    <el-dialog v-model="showInstallDialog" title="安装 Mod" width="500px" align-center custom-class="glass-dialog">
        <el-form label-width="100px" :model="installForm">
            <el-form-item label="Mod 名称">
                <el-input v-model="installForm.modName" placeholder="建议使用英文" />
            </el-form-item>
            <el-form-item label="分组/角色">
                 <el-autocomplete
                    v-model="installForm.targetGroup"
                    :fetch-suggestions="(qs, cb) => cb(groups.filter((g: any) => g.id !== 'All' && g.id.toLowerCase().includes(qs.toLowerCase())).map((x: any) => ({ value: x.id })))"
                    placeholder="输入角色名或路径(如 A/B)"
                    style="width: 100%"
                >
                    <template #default="{ item }">
                         <span>{{ item.value }}</span>
                    </template>
                </el-autocomplete>
            </el-form-item>
            
            <el-divider>文件预览</el-divider>
            <div class="preview-info" v-if="installPreview">
                <p><strong>格式:</strong> {{ installPreview.format.toUpperCase() }}</p>
                <p><strong>包含文件数:</strong> {{ installPreview.file_count }}</p>
                <p><strong>根目录文件夹:</strong> {{ installPreview.root_dirs.join(', ') || '无 (直接包含文件)' }}</p>
                <p v-if="installPreview.has_ini" style="color: #67c23a"><el-icon><Refresh /></el-icon> 检测到 .ini 文件 (这是一个有效的 Mod)</p>
                <p v-else style="color: #e6a23c">未检测到 .ini 文件，可能是素材包？</p>
            </div>
            <!-- Password prompt if needed in future 
            <el-form-item label="解压密码" v-if="needed">
                 <el-input v-model="installForm.password" />
            </el-form-item> 
            -->
        </el-form>
        <template #footer>
            <span class="dialog-footer">
                <el-button @click="showInstallDialog = false">取消</el-button>
                <el-button type="primary" @click="confirmInstall" :loading="isInstalling">
                    确认安装
                </el-button>
            </span>
        </template>
    </el-dialog>

    <!-- Custom Context Menu -->
    <div 
        v-if="contextMenu.visible"
        class="custom-context-menu glass-panel"
        :style="{ top: contextMenu.y + 'px', left: contextMenu.x + 'px' }"
        @click.stop
    >
        <div v-if="contextMenu.type === 'mod'" class="menu-content">
            <div class="menu-header">{{ contextMenu.target?.name }}</div>
            <div class="menu-divider"></div>
            <div class="menu-item has-submenu">
                <el-icon><FolderAdd /></el-icon>
                <span>移动到...</span>
                <el-icon class="arrow-right"><ArrowRight /></el-icon>
                
                <div class="submenu glass-panel">
                    <div class="menu-item" @click="moveModToGroup(contextMenu.target, 'Root')">
                        <span>未分类 (Root)</span>
                    </div>
                    <div 
                        v-for="group in groups.filter((g: any) => g.id !== 'All' && g.id !== 'Root')" 
                        :key="group.id"
                        class="menu-item"
                        @click="moveModToGroup(contextMenu.target, group.id)"
                    >
                        <span>{{ group.id }}</span>
                    </div>
                    <div class="menu-divider"></div>
                     <div class="menu-item" @click="createNewGroup">
                        <el-icon><Plus /></el-icon>
                        <span>新建分类...</span>
                    </div>
                </div>
            </div>
        </div>

         <div v-if="contextMenu.type === 'group'" class="menu-content">
            <div class="menu-header">{{ contextMenu.target.split('/').pop() }}</div>
            <div class="menu-divider"></div>
            <div class="menu-item" @click="openModGroupFolder(contextMenu.target)">
                <el-icon><Folder /></el-icon>
                <span>打开文件夹</span>
            </div>
            <div class="menu-item" @click="setGroupIcon(contextMenu.target)">
                <el-icon><Picture /></el-icon>
                <span>设置图标</span>
            </div>
            <div class="menu-item" @click="renameGroup(contextMenu.target)">
                <el-icon><Edit /></el-icon>
                <span>重命名</span>
            </div>
             <div class="menu-item" @click="deleteGroup(contextMenu.target)" style="color: #ff4949">
                <el-icon><Delete /></el-icon>
                <span>删除</span>
            </div>
        </div>
    </div>
  </div>
</template>

<style scoped>
.page-container.mod-manager {
    height: 100%;
    display: flex;
    flex-direction: column;
    padding: 0;
    overflow: hidden;
}

/* Glass Panel Utility */
.glass-panel {
    background: rgba(30, 30, 35, 0.6);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.2);
}

/* Toolbar */
.toolbar {
    padding: 12px 24px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    z-index: 10;
    flex-shrink: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.left-tools, .right-tools {
    display: flex;
    align-items: center;
    gap: 12px;
}

.divider-vertical {
    width: 1px;
    height: 24px;
    background: rgba(255, 255, 255, 0.2);
    margin: 0 8px;
}

/* Main Layout */
.main-content {
    flex: 1;
    display: flex;
    overflow: hidden;
}

/* Sidebar */
.sidebar {
    width: 220px;
    flex-shrink: 0;
    overflow-y: auto;
    border-right: 1px solid rgba(255, 255, 255, 0.05);
    background: rgba(20, 20, 25, 0.4); /* Slightly dark */
}

.group-list {
    padding: 12px;
}

.group-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    margin-bottom: 4px;
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    color: #a0a0a0;
}

.group-item:hover {
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
}

.group-item.active {
    background: rgba(64, 158, 255, 0.2);
    color: #409eff;
    font-weight: 500;
}

.group-icon {
    width: 20px;
    height: 20px;
    margin-right: 6px;
    border-radius: 4px;
    overflow: hidden;
    flex-shrink: 0;
}
.icon-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.group-name {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.count {
    font-size: 12px;
    background: rgba(0, 0, 0, 0.2);
    padding: 2px 6px;
    border-radius: 10px;
}

/* Mod Grid */
.mod-grid-container {
    flex: 1;
    overflow-y: auto;
    padding: 24px;
    /* Custom Scrollbar */
}

/* Scrollbar styling for webkit */
.mod-grid-container::-webkit-scrollbar,
.sidebar::-webkit-scrollbar {
    width: 8px;
}
.mod-grid-container::-webkit-scrollbar-track,
.sidebar::-webkit-scrollbar-track {
    background: transparent;
}
.mod-grid-container::-webkit-scrollbar-thumb,
.sidebar::-webkit-scrollbar-thumb {
    background-color: rgba(255, 255, 255, 0.2);
    border-radius: 4px;
}

.mod-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
    gap: 20px;
}

.mod-card {
    border-radius: 12px;
    overflow: hidden;
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
    display: flex;
    flex-direction: column;
    height: 260px;
    background: rgba(30, 30, 35, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.05);
    position: relative;
}

.mod-card:hover {
    transform: translateY(-6px);
    box-shadow: 0 12px 24px rgba(0, 0, 0, 0.5);
    border-color: rgba(255, 255, 255, 0.2);
    z-index: 2;
}

/* Disabled State Visuals */
.mod-card.is-disabled {
    opacity: 0.85;
}
.mod-card.is-disabled .image-wrapper {
    filter: grayscale(1) contrast(0.8) brightness(0.8);
    transition: filter 0.3s;
}
.mod-card.is-disabled:hover .image-wrapper {
    filter: grayscale(0.5);
}

.card-preview {
    flex: 1;
    position: relative;
    background: #000;
    overflow: hidden;
}

.image-wrapper {
    width: 100%;
    height: 100%;
    transition: transform 0.5s ease;
}
.mod-card:hover .image-wrapper {
    transform: scale(1.05);
}

.image-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #1e1e24, #141417);
    color: rgba(255, 255, 255, 0.2);
    font-size: 48px;
    font-weight: 800;
}
.preview-info {
    font-size: 13px;
    color: #ccc;
    background: rgba(0,0,0,0.2);
    padding: 10px;
    border-radius: 4px;
}
.preview-info p {
    margin: 4px 0;
}

.char-avatar {
    text-transform: uppercase;
}

/* Hover Action Overlay */
.card-overlay {
    position: absolute;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s;
}
.mod-card:hover .card-overlay {
    opacity: 1;
}

/* Footer Info Area */
.card-info {
    padding: 12px 14px;
    background: rgba(18, 18, 20, 0.95);
    border-top: 1px solid rgba(255, 255, 255, 0.05);
    height: auto;
    min-height: 64px;
    display: flex;
    flex-direction: column;
    justify-content: center;
}

.header-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 10px;
}

.text-content {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
    gap: 2px;
}

.mod-name {
    font-weight: 600;
    color: #f0f0f0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 14px;
    letter-spacing: 0.3px;
}

.mod-group {
    font-size: 11px;
    color: #666;
    display: flex;
    align-items: center;
    gap: 4px;
}
/*.mod-group::before {
    content: '';
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background-color: #666;
}*/
.mini-group-icon {
    width: 14px;
    height: 14px;
    border-radius: 2px;
    object-fit: cover;
}

/* Active dot color if mod is enabled? Could be cool */
/*
.mod-card:not(.is-disabled) .mod-group::before {
    background-color: #67C23A;
    box-shadow: 0 0 6px rgba(103, 194, 58, 0.5);
}
*/


/* Switch styling tweak */
:deep(.el-switch__core) {
    background-color: rgba(255,255,255,0.1);
    border-color: transparent;
}

/* Tree Styles */
.group-tree {
    background: transparent; 
    color: #cfcfcf;
}
:deep(.el-tree-node__content) {
    height: 36px;
    border-radius: 4px;
    margin-bottom: 2px;
}
:deep(.el-tree-node__content:hover) {
    background-color: rgba(255, 255, 255, 0.08) !important;
}
:deep(.el-tree--highlight-current .el-tree-node.is-current > .el-tree-node__content) {
    background-color: rgba(64, 158, 255, 0.15) !important;
    color: #409eff;
}
:deep(.el-tree-node__expand-icon) {
    color: rgba(255, 255, 255, 0.4);
}
:deep(.el-tree-node__expand-icon.is-leaf) {
    color: transparent;
}

.custom-tree-node {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding-right: 8px;
    overflow: hidden;
}

.node-content {
    display: flex;
    align-items: center;
    gap: 8px;
    overflow: hidden;
}

.node-label {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    font-size: 13px;
}

.tree-icon {
    width: 20px;
    height: 20px;
    object-fit: cover;
    border-radius: 4px;
}

.tree-icon-placeholder {
    font-size: 16px;
    color: #888;
}
:deep(.el-switch.is-checked .el-switch__core) {
    background-color: #67C23A;
}

/* Context Menu */
.custom-context-menu {
    position: fixed;
    z-index: 9999;
    background: rgba(30, 30, 30, 0.95);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 4px 0;
    min-width: 160px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.menu-header {
    padding: 8px 16px;
    font-size: 0.85em;
    color: rgba(255, 255, 255, 0.5);
    font-weight: 600;
    border-bottom: 1px solid rgba(255, 255, 255, 0.05);
    margin-bottom: 4px;
}

.menu-item {
    padding: 8px 16px;
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: background 0.2s;
    color: #eee;
    font-size: 0.9em;
    position: relative;
}

.menu-item:hover {
    background: rgba(255, 255, 255, 0.1);
}

.menu-divider {
    height: 1px;
    background: rgba(255, 255, 255, 0.1);
    margin: 4px 0;
}

.has-submenu .submenu {
    display: none;
    position: absolute;
    left: 100%;
    top: 0;
    margin-left: 4px;
    /* Reuse base styles */
    background: rgba(30, 30, 30, 0.95);
    backdrop-filter: blur(10px);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    padding: 4px 0;
    min-width: 160px;
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
}

.has-submenu:hover .submenu {
    display: block;
}

.arrow-right {
    margin-left: auto;
    font-size: 0.8em;
    opacity: 0.7;
}

.sidebar-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 16px 20px 8px 20px;
    margin-bottom: 8px;
}
.sidebar-header .title {
    font-weight: 600;
    color: #fff;
    font-size: 1.1em;
}
</style>
