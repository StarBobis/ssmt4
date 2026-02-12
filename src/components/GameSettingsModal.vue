<script setup lang="ts">
import { ref, watch, reactive, computed } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { ask, open, message } from '@tauri-apps/plugin-dialog';
// import { openPath } from '@tauri-apps/plugin-opener'; // Updated import
import { join } from '@tauri-apps/api/path';
import { loadGames, appSettings, gamesList, switchToGame } from '../store'; // Need to reload games list to see new configs
import { useI18n } from 'vue-i18n';

const { t } = useI18n();

const props = defineProps<{
  modelValue: boolean;
  gameName: string;
}>();

const emit = defineEmits<{
  (e: 'update:modelValue', value: boolean): void;
}>();

// Config State
interface GameConfig {
  basic: {
    // configName is kept in UI state but separate from the object sent to backend
    gamePreset: string;
    backgroundType?: 'Image' | 'Video';
  };
  threeDMigoto: {
    installDir: string;
    targetExePath: string;
    launcherExePath: string;
    launchArgs: string;
    showErrorPopup: boolean;
    autoSetAnalyseOptions: boolean;
    useShell: boolean;
    useUpx: boolean;
    delay: number;
    autoExitSeconds: number;
    extraDll: string;
  };
  other: any;
}

const config = reactive<GameConfig>({
  basic: { gamePreset: 'GIMI', backgroundType: 'Image' },
  threeDMigoto: {
    installDir: '',
    targetExePath: '',
    launcherExePath: '',
    launchArgs: '',
    showErrorPopup: true,
    autoSetAnalyseOptions: true,
    useShell: false,
    useUpx: false,
    delay: 0,
    autoExitSeconds: 0,
    extraDll: ''
  },
  other: {}
});

const configName = ref(''); // Separate UI state for the folder name


const isLoading = ref(false);

// Tabs
const activeTab = ref('basic');
const tabs = computed(() => [
  { id: 'basic', label: t('gamesettingsmodal.basicsettings') },
  { id: '3dmigoto', label: t('gamesettingsmodal.migoto') },
  { id: 'other', label: t('gamesettingsmodal.other') },
]);

const presetOptions = [
  { label: 'GIMI', value: 'GIMI' },
  { label: 'HIMI', value: 'HIMI' },
  { label: 'SRMI', value: 'SRMI' },
  { label: 'ZZMI', value: 'ZZMI' },
  { label: 'WWMI', value: 'WWMI' },
  { label: 'EFMI', value: 'EFMI' },
  { label: 'GF2', value: 'GF2' },
  { label: 'IdentityVNeoX2', value: 'IdentityVNeoX2' },
  { label: 'IdentityVNeoX3', value: 'IdentityVNeoX3' },
  { label: 'AILIMIT', value: 'AILIMIT' },
  { label: 'DOAV', value: 'DOAV' },
  { label: 'MiSide', value: 'MiSide' },
  { label: 'SnowBreak', value: 'SnowBreak' },
  { label: 'Strinova', value: 'Strinova' },
  { label: 'Nioh2', value: 'Nioh2' },
  { label: 'YYSLS', value: 'YYSLS' },
  { label: 'WuWa', value: 'WuWa' },
  { label: 'AEMI', value: 'AEMI' },
  { label: 'DIY', value: 'DIY' },
];

// Load/Save Logic
const loadConfig = async () => {
  if (!props.gameName) return;
  isLoading.value = true;
  try {
    const data = await invoke<GameConfig>('load_game_config', { gameName: props.gameName });
    // Merge
    config.basic = {
      gamePreset: data.basic.gamePreset || 'GIMI',
      backgroundType: (data.basic as any).backgroundType || 'Image'
    };

    const threeDMigotoData = data.threeDMigoto || {};
    config.threeDMigoto = {
      installDir: threeDMigotoData.installDir || '',
      targetExePath: threeDMigotoData.targetExePath || '',
      launcherExePath: threeDMigotoData.launcherExePath || '',
      launchArgs: threeDMigotoData.launchArgs || '',
      showErrorPopup: threeDMigotoData.showErrorPopup !== undefined ? threeDMigotoData.showErrorPopup : true,
      autoSetAnalyseOptions: threeDMigotoData.autoSetAnalyseOptions !== undefined ? threeDMigotoData.autoSetAnalyseOptions : true,
      useShell: threeDMigotoData.useShell || false,
      useUpx: threeDMigotoData.useUpx || false,
      delay: threeDMigotoData.delay !== undefined ? threeDMigotoData.delay : 100,
      autoExitSeconds: threeDMigotoData.autoExitSeconds !== undefined ? threeDMigotoData.autoExitSeconds : 5,
      extraDll: threeDMigotoData.extraDll || ''
    };

    // Default Logic for installDir if empty on first load (user requirement)
    if (!config.threeDMigoto.installDir && appSettings.cacheDir) {
      try {
        // "SSMT Cache/3Dmigoto/GameName"
        config.threeDMigoto.installDir = await join(appSettings.cacheDir, '3Dmigoto', props.gameName);
      } catch (err) {
        console.error(t('gamesettingsmodal.error.failconstructdefaultpath'), err);
      }
    }

    config.other = data.other || {};
    // Note: configName is NOT set from file, but from props
  } catch (e) {
    console.error(t('gamesettingsmodal.error.failloadconfig'), e);
  } finally {
    isLoading.value = false;
  }
};

const saveConfig = async () => {
  if (!props.gameName || isLoading.value) return; // Prevent saving if loading isn't complete
  console.log(t('gamesettingsmodal.log.saving', { gameName: props.gameName }));
  console.log(t('gamesettingsmodal.log.currentstate', { currentState: JSON.parse(JSON.stringify(config)) }));

  try {
    await invoke('save_game_config', {
      gameName: props.gameName,
      config: config
    });
    console.log(t('gamesettingsmodal.log.configsaved'));
  } catch (e) {
    console.error(t('gamesettingsmodal.error.configfailedsaving'), { e: e });
  }
};

const handleBgTypeChange = async () => {
  await saveConfig();
  // Refresh global state if this is the active game
  if (appSettings.currentConfigName === props.gameName) {
    await loadGames();
  }
};

const selectIcon = async () => {
  try {
    const file = await open({
      multiple: false,
      filters: [{ name: 'Images', extensions: ['png'] }]
    });

    if (file) {
      await invoke('set_game_icon', {
        gameName: props.gameName,
        filePath: file
      });
      await loadGames();
    }
  } catch (e) {
    console.error(e);
  }
};

const selectBackground = async () => {
  try {
    const isVideo = config.basic.backgroundType === 'Video';
    const filters = isVideo
      ? [{ name: 'Videos', extensions: ['mp4', 'webm', 'ogg', 'mov'] }]
      : [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'gif', 'svg', 'bmp', 'ico', 'avif'] }];

    const file = await open({
      multiple: false,
      filters
    });

    if (file) {
      await invoke('set_game_background', {
        gameName: props.gameName,
        filePath: file,
        bgType: config.basic.backgroundType
      });
      await loadGames(); // Refresh
    }
  } catch (e) {
    console.error(e);
  }
};

const autoSupportedPresets = ['GIMI', 'HIMI', 'SRMI', 'ZZMI'];
const canAutoUpdate = computed(() => autoSupportedPresets.includes(config.basic.gamePreset));

const autoUpdateBackground = async () => {
  try {
    isLoading.value = true;
    await invoke('update_game_background', {
      gameName: props.gameName,
      gamePreset: config.basic.gamePreset,
      bgType: config.basic.backgroundType
    });
    await loadGames();
    await message(
      t('gamesettingsmodal.message.success.backgroundupdated'),
      {
        title: t('gamesettingsmodal.message.success.title'),
        kind: 'info'
      }
    );

    if (appSettings.currentConfigName === props.gameName) {
      // Force refresh UI if active
      const current = gamesList.find(g => g.name === props.gameName);
      if (current) switchToGame(current);
    }

  } catch (e) {
    console.error(e);

    await message(
      t('gamesettingsmodal.message.error.updateFailed', { error: e }),
      {
        title: t('gamesettingsmodal.message.error.title'),
        kind: 'error'
      }
    );
  } finally {
    isLoading.value = false;
  }
};

// 3Dmigoto Helper Functions
const pick3dmigotoDir = async () => {
  try {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择3Dmigoto所在目录'
    });
    if (selected && typeof selected === 'string') {
      config.threeDMigoto.installDir = selected;
    }
  } catch (e) { console.error(e); }
};

const open3dmigotoDir = async () => {
  if (config.threeDMigoto.installDir) {
    try {
      await invoke('ensure_directory', { path: config.threeDMigoto.installDir });
      await invoke('open_in_explorer', { path: config.threeDMigoto.installDir });
    } catch (e) {
      console.error(t('gamesettingsmodal.error.failedopendir'), { e: e });
    }
  }
};

const packageSupportedPresets = ['GIMI', 'HIMI', 'SRMI', 'ZZMI', 'WWMI', 'EFMI', 'AEMI'];
const canUpdatePackage = computed(() => packageSupportedPresets.includes(config.basic.gamePreset));

const check3DMigotoPackageUpdate = async () => {
  // 1. Initial Confirmation
  const checkConfirm = await ask(
    t('gamesettingsmodal.confirm.checkUpdate.message', {
      gamePreset: config.basic.gamePreset
    }),
    {
      title: t('gamesettingsmodal.confirm.checkUpdate.title'),
      kind: 'info'
    }
  );
  if (!checkConfirm) return;

  try {
    isLoading.value = true;

    // 2. Fetch Info
    const info = await invoke<{ version: string, description: string, downloadUrl: string }>('get_3dmigoto_latest_release', {
      gamePreset: config.basic.gamePreset
    });

    isLoading.value = false;

    // 3. Show info and ask for second confirmation
    const updateConfirm = await ask(
      t('gamesettingsmodal.confirm.versionUpdate.message', {
        version: info.version,
        description: info.description
      }),
      {
        title: t('gamesettingsmodal.confirm.versionUpdate.title'),
        kind: 'info'
      }
    );

    if (!updateConfirm) return;

    // 4. Perform Update
    isLoading.value = true;
    await invoke('install_3dmigoto_update', {
      gameName: props.gameName,
      downloadUrl: info.downloadUrl
    });

    await message(
      t('gamesettingsmodal.message.success.updatedToVersion', {
        version: info.version
      }),
      {
        title: t('gamesettingsmodal.message.success.title'),
        kind: 'info'
      }
    );

  } catch (e) {
    console.error(e);
    await message(
      t('gamesettingsmodal.message.error.operationFailed', { error: e }),
      {
        title: t('gamesettingsmodal.message.error.title'),
        kind: 'error'
      }
    );
  } finally {
    isLoading.value = false;
  }
};

const pickExe = async (field: 'targetExePath' | 'launcherExePath') => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ 
        name: t('gamesettingsmodal.filePicker.filterName'), 
        extensions: ['exe'] 
      }],
      title: t('gamesettingsmodal.filePicker.title')
    });
    if (selected && typeof selected === 'string') {
      config.threeDMigoto[field] = selected;
    }
  } catch (e) { console.error(e); }
};

const openExeDir = async (field: 'targetExePath' | 'launcherExePath') => {
  const path = config.threeDMigoto[field];
  if (path) {
    try {
      const lastSlash = Math.max(path.lastIndexOf('/'), path.lastIndexOf('\\'));
      if (lastSlash > -1) {
        const dir = path.substring(0, lastSlash);
        await invoke('ensure_directory', { path: dir });
        await invoke('open_in_explorer', { path: dir });
      }
    } catch (e) { console.error(e); }
  }
};

const pickDll = async () => {
  try {
    const selected = await open({
      multiple: false,
      filters: [{ 
        name: t('gamesettingsmodal.filePicker.dllFilterName'), 
        extensions: ['dll'] 
      }],
      title: t('gamesettingsmodal.filePicker.dllTitle')
    });
    if (selected && typeof selected === 'string') {
      config.threeDMigoto.extraDll = selected;
    }
  } catch (e) { console.error(e); }
};

const setDefaultDll = async () => {
  if (config.threeDMigoto.installDir) {
    try {
      const dllPath = await join(config.threeDMigoto.installDir, 'd3d11.dll');
      config.threeDMigoto.extraDll = dllPath;
    } catch (e) {
      console.error(t('gamesettingsmodal.log.joinPathFailed', { error: e }));
    }
  }
};

const createNewConfig = async () => {
  if (!configName.value) return;

  const yes = await ask(
    t('gamesettingsmodal.confirm.createConfig.message', { 
      configName: configName.value 
    }),
    {
      title: t('gamesettingsmodal.confirm.createConfig.title'),
      kind: 'info',
    }
  );
  if (!yes) return;

  try {
    isLoading.value = true;
    await invoke('create_new_config', {
      newName: configName.value,
      config: config
    });
    console.log(t('gamesettingsmodal.log.configCreated', { 
      configName: configName.value 
    }));

    // Refresh games list and close
    await loadGames();

    // Switch to the newly created game
    const newGame = gamesList.find(g => g.name === configName.value);
    if (newGame) {
      switchToGame(newGame);
    }

    close();
  } catch (e) {
    console.error(t('gamesettingsmodal.log.configCreateFailed', { error: e }));
  } finally {
    isLoading.value = false;
  }
};

const deleteCurrentConfig = async () => {
  if (!props.gameName) return;

  const yes = await ask(`确定要删除配置 "${props.gameName}" 吗？此操作不可逆。`, {
    title: '删除确认',
    kind: 'warning',
  });
  if (!yes) return;

  try {
    isLoading.value = true;
    await invoke('delete_game_config_folder', { gameName: props.gameName });
    console.log('Deleted config:', props.gameName);

    // Refresh games list and close
    await loadGames();
    close();
  } catch (e) {
    console.error('Failed to delete config:', e);
  } finally {
    isLoading.value = false;
  }
};

// Open/Close
watch(() => props.modelValue, (val) => {
  if (val) {
    activeTab.value = 'basic'; // Reset to first tab
    configName.value = props.gameName; // Initialize config name from current game
    loadConfig();
  } else {
    // When closing, save
    saveConfig();
  }
});

const close = () => {
  emit('update:modelValue', false);
};

// Expose methods to parent
defineExpose({
  switchTab: (tabId: string) => {
    activeTab.value = tabId;
  },
  runPackageUpdate: () => {
    // Ensure we are on the right tab visually
    activeTab.value = '3dmigoto';
    // Run the update check
    if (canUpdatePackage.value) {
      check3DMigotoPackageUpdate();
    } else {
      message('当前预设不支持自动更新整合包', { title: '提示', kind: 'info' });
    }
  }
});
</script>

<template>
  <transition name="modal-fade">
    <div v-if="modelValue" class="settings-overlay">
      <div class="settings-window">
        <!-- Loading Overlay -->
        <div v-if="isLoading" class="loading-overlay">
          <div class="spinner"></div>
          <div class="loading-text">处理中...</div>
        </div>

        <!-- Sidebar -->
        <div class="settings-sidebar">
          <div class="sidebar-title">游戏设置</div>

          <div v-for="tab in tabs" :key="tab.id" class="sidebar-item" :class="{ active: activeTab === tab.id }"
            @click="activeTab = tab.id">
            {{ tab.label }}
          </div>
        </div>

        <!-- Content Area -->
        <div class="settings-content">
          <div class="content-header">
            <span class="header-title">{{tabs.find(t => t.id === activeTab)?.label}}</span>
            <div class="close-btn" @click="close">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none"
                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </div>
          </div>

          <div class="scroll-content">
            <!-- Basic Settings -->
            <div v-if="activeTab === 'basic'" class="tab-pane">
              <div class="setting-group">
                <div class="setting-label">配置名称 (文件夹名)</div>
                <input v-model="configName" type="text" class="custom-input" placeholder="请输入配置名称" />

                <div class="button-row">
                  <button class="action-btn create" @click="createNewConfig">
                    以此名称创建新配置
                  </button>
                  <button class="action-btn delete" @click="deleteCurrentConfig">
                    删除当前配置
                  </button>
                </div>
              </div>

              <div class="setting-group">
                <div class="setting-label">游戏预设</div>
                <el-select v-model="config.basic.gamePreset" placeholder="Select" class="custom-select"
                  @change="saveConfig">
                  <el-option v-for="item in presetOptions" :key="item.value" :label="item.label" :value="item.value" />
                </el-select>
              </div>

              <div class="setting-group">
                <div class="setting-label">游戏图标</div>
                <button class="action-btn" @click="selectIcon">选择图标文件 (.png)</button>
              </div>

              <div class="setting-group">
                <div class="setting-label">背景设置</div>
                <div style="margin-bottom: 10px;">
                  <el-radio-group v-model="config.basic.backgroundType" @change="handleBgTypeChange">
                    <el-radio value="Image" label="Image">图片</el-radio>
                    <el-radio value="Video" label="Video">视频</el-radio>
                  </el-radio-group>
                </div>
                <!-- Separate check: if video, show video file btn, if image, show image file btn -->
                <div class="button-row">
                  <button class="action-btn" @click="selectBackground">
                    {{ config.basic.backgroundType === 'Video' ? '选择背景视频' : '选择背景图片' }}
                  </button>
                  <button v-if="canAutoUpdate" class="action-btn" @click="autoUpdateBackground">
                    自动更新背景
                  </button>
                </div>
              </div>
            </div>

            <!-- 3Dmigoto Settings -->
            <div v-if="activeTab === '3dmigoto'" class="tab-pane">

              <div class="setting-group">
                <div class="setting-label">3Dmigoto 目录</div>
                <input v-model="config.threeDMigoto.installDir" type="text" class="custom-input"
                  placeholder="请选择或输入目录" />
                <div class="button-row">
                  <button class="action-btn" @click="pick3dmigotoDir">选择文件夹</button>
                  <button class="action-btn" @click="open3dmigotoDir">打开文件夹</button>
                  <button v-if="canUpdatePackage" class="action-btn highlight"
                    @click="check3DMigotoPackageUpdate">更新整合包</button>
                </div>
              </div>

              <div class="setting-group">
                <div class="setting-label">目标进程路径 (Target Exe)</div>
                <input v-model="config.threeDMigoto.targetExePath" type="text" class="custom-input"
                  placeholder="选择游戏主程序" />
                <div class="button-row">
                  <button class="action-btn" @click="pickExe('targetExePath')">选择文件</button>
                  <button class="action-btn" @click="openExeDir('targetExePath')">打开所在位置</button>
                </div>
              </div>

              <div class="setting-group">
                <div class="setting-label">启动文件路径 (Launcher Exe)</div>
                <input v-model="config.threeDMigoto.launcherExePath" type="text" class="custom-input"
                  placeholder="选择启动器 (可选)" />
                <div class="button-row">
                  <button class="action-btn" @click="pickExe('launcherExePath')">选择文件</button>
                  <button class="action-btn" @click="openExeDir('launcherExePath')">打开所在位置</button>
                </div>
              </div>

              <div class="setting-group">
                <div class="setting-label">启动参数</div>
                <input v-model="config.threeDMigoto.launchArgs" type="text" class="custom-input"
                  placeholder="例如: -popupwindow" />
              </div>

              <div class="setting-checkbox-row">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="config.threeDMigoto.showErrorPopup" />
                  显示左上角报错 (show_warnings)
                </label>
              </div>

              <div class="setting-checkbox-row">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="config.threeDMigoto.autoSetAnalyseOptions" />
                  自动设置 analyse_options
                </label>
              </div>

              <div class="setting-checkbox-row">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="config.threeDMigoto.useShell" />
                  使用 Shell 方式运行
                </label>
              </div>

              <div class="setting-checkbox-row">
                <label class="checkbox-label">
                  <input type="checkbox" v-model="config.threeDMigoto.useUpx" />
                  使用 UPX 默认选项加壳
                </label>
              </div>

              <div class="flex-row">
                <div class="setting-group half-width">
                  <div class="setting-label">d3d11.dll 延迟 (Delay ms)</div>
                  <input v-model.number="config.threeDMigoto.delay" type="number" class="custom-input" />
                </div>
                <div class="setting-group half-width">
                  <div class="setting-label">自动退出秒数 (Auto Exit s)</div>
                  <input v-model.number="config.threeDMigoto.autoExitSeconds" type="number" class="custom-input" />
                </div>
              </div>

              <div class="setting-group">
                <div class="setting-label">额外注入 DLL</div>
                <input v-model="config.threeDMigoto.extraDll" type="text" class="custom-input" placeholder="选择或者留空" />
                <div class="button-row">
                  <button class="action-btn" @click="pickDll">选择文件</button>
                  <button class="action-btn highlight" @click="setDefaultDll">设为当前d3d11.dll</button>
                </div>
              </div>

            </div>

            <!-- Other Settings -->
            <div v-if="activeTab === 'other'" class="tab-pane">
              <div class="empty-state">暂无设置项</div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.settings-overlay {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  background-color: rgba(0, 0, 0, 0.5);
  backdrop-filter: blur(4px);
  z-index: 2000;
  /* High z-index */
}

.settings-window {
  position: absolute;
  top: 50px;
  bottom: 60px;
  left: 100px;
  right: 100px;
  background: rgba(30, 30, 30, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  border-radius: 12px;
  display: flex;
  overflow: hidden;
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }

  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.setting-checkbox-row {
  margin-bottom: 12px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  color: white;
  cursor: pointer;
  user-select: none;
}

.flex-row {
  display: flex;
  gap: 16px;
}

.half-width {
  flex: 1;
}

/* Sidebar */
.settings-sidebar {
  width: 200px;
  background: rgba(0, 0, 0, 0.2);
  border-right: 1px solid rgba(255, 255, 255, 0.05);
  display: flex;
  flex-direction: column;
  padding: 20px 0;
}

.sidebar-title {
  font-size: 16px;
  font-weight: bold;
  color: rgba(255, 255, 255, 0.9);
  padding: 0 20px 20px 20px;
  margin-bottom: 10px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.sidebar-item {
  padding: 12px 20px;
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.sidebar-item:hover {
  background: rgba(255, 255, 255, 0.05);
  color: #fff;
}

.sidebar-item.active {
  background: rgba(247, 206, 70, 0.1);
  /* Yellow tint */
  color: #F7CE46;
  border-left: 3px solid #F7CE46;
}

/* Content */
.settings-content {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.content-header {
  height: 60px;
  padding: 0 30px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.header-title {
  font-size: 18px;
  font-weight: 600;
  color: #fff;
}

.close-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.6);
  transition: all 0.2s;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.scroll-content {
  flex: 1;
  padding: 30px;
  overflow-y: auto;
}

.setting-group {
  margin-bottom: 24px;
}

.setting-label {
  display: block;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
  margin-bottom: 8px;
}

.custom-input {
  width: 100%;
  box-sizing: border-box;
  background: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  padding: 8px 12px;
  color: #fff;
  font-size: 14px;
  outline: none;
  transition: border-color 0.2s;
}

.custom-input:focus {
  border-color: #F7CE46;
}

.button-row {
  display: flex;
  gap: 12px;
  margin-top: 12px;
}

.action-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 4px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  flex: 1;
  color: #fff;
  background: rgba(255, 255, 255, 0.1);
}

.action-btn:hover {
  background: rgba(255, 255, 255, 0.2);
}

.action-btn.create {
  background: rgba(247, 206, 70, 0.2);
  border: 1px solid rgba(247, 206, 70, 0.4);
  color: #F7CE46;
}

.action-btn.create:hover {
  background: rgba(247, 206, 70, 0.3);
}

.action-btn.highlight {
  background: rgba(0, 122, 204, 0.3);
  border: 1px solid rgba(0, 122, 204, 0.5);
  color: #61afef;
}

.action-btn.highlight:hover {
  background: rgba(0, 122, 204, 0.5);
}

.action-btn.delete {
  background: rgba(232, 17, 35, 0.2);
  border: 1px solid rgba(232, 17, 35, 0.4);
  color: #ff6b6b;
}

.action-btn.delete:hover {
  background: rgba(232, 17, 35, 0.3);
}

.empty-state {
  color: rgba(255, 255, 255, 0.3);
  text-align: center;
  margin-top: 40px;
}

/* Transitions */
.modal-fade-enter-active,
.modal-fade-leave-active {
  transition: opacity 0.3s ease;
}

.modal-fade-enter-from,
.modal-fade-leave-to {
  opacity: 0;
}

.loading-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  z-index: 100;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  backdrop-filter: blur(2px);
  border-radius: 12px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid rgba(255, 255, 255, 0.1);
  border-top-color: #F7CE46;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin-bottom: 12px;
}

.loading-text {
  color: #F7CE46;
  font-size: 14px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}
</style>
