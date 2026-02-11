<script setup lang="ts">
import { ref, watch, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';

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
    gamePreset: string;
  };
  threeDMigoto: any;
  other: any;
}

const config = reactive<GameConfig>({
  basic: { gamePreset: 'Default' },
  threeDMigoto: {},
  other: {}
});

const isLoading = ref(false);

// Tabs
const activeTab = ref('basic');
const tabs = [
  { id: 'basic', label: '基础设置' },
  { id: '3dmigoto', label: '3Dmigoto设置' },
  { id: 'other', label: '其他设置' },
];

const presetOptions = [
  { label: '预设 (Default)', value: 'Default' },
  { label: '高画质 (High)', value: 'High' },
  { label: '低画质 (Low)', value: 'Low' },
  { label: '自定义 (Custom)', value: 'Custom' },
];

// Load/Save Logic
const loadConfig = async () => {
  if (!props.gameName) return;
  isLoading.value = true;
  try {
    const data = await invoke<GameConfig>('load_game_config', { gameName: props.gameName });
    // Merge
    config.basic = data.basic || { gamePreset: 'Default' };
    config.threeDMigoto = data.threeDMigoto || {};
    config.other = data.other || {};
  } catch (e) {
    console.error('Failed to load game config:', e);
  } finally {
    isLoading.value = false;
  }
};

const saveConfig = async () => {
  if (!props.gameName || isLoading.value) return; // Prevent saving if loading isn't complete
  try {
    await invoke('save_game_config', { 
      gameName: props.gameName, 
      config: config 
    });
    console.log('Game config saved');
  } catch (e) {
    console.error('Failed to save game config:', e);
  }
};

// Open/Close
watch(() => props.modelValue, (val) => {
  if (val) {
    activeTab.value = 'basic'; // Reset to first tab
    loadConfig();
  } else {
    // When closing, save
    saveConfig();
  }
});

const close = () => {
  emit('update:modelValue', false);
};
</script>

<template>
  <transition name="modal-fade">
    <div v-if="modelValue" class="settings-overlay" @click.self="close">
      <div class="settings-window">
        <!-- Sidebar -->
        <div class="settings-sidebar">
          <div class="sidebar-title">游戏设置</div>
          
          <div 
            v-for="tab in tabs" 
            :key="tab.id"
            class="sidebar-item"
            :class="{ active: activeTab === tab.id }"
            @click="activeTab = tab.id"
          >
            {{ tab.label }}
          </div>
        </div>

        <!-- Content Area -->
        <div class="settings-content">
          <div class="content-header">
            <span class="header-title">{{ tabs.find(t => t.id === activeTab)?.label }}</span>
            <div class="close-btn" @click="close">
              <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <line x1="18" y1="6" x2="6" y2="18"></line>
                <line x1="6" y1="6" x2="18" y2="18"></line>
              </svg>
            </div>
          </div>

          <div class="scroll-content">
            <!-- Basic Settings -->
            <div v-if="activeTab === 'basic'" class="tab-pane">
              <div class="setting-group">
                <div class="setting-label">游戏预设</div>
                <el-select v-model="config.basic.gamePreset" placeholder="Select" class="custom-select">
                  <el-option
                    v-for="item in presetOptions"
                    :key="item.value"
                    :label="item.label"
                    :value="item.value"
                  />
                </el-select>
              </div>
            </div>

            <!-- 3Dmigoto Settings -->
            <div v-if="activeTab === '3dmigoto'" class="tab-pane">
              <div class="empty-state">暂无设置项</div>
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
  z-index: 2000; /* High z-index */
  display: flex;
  justify-content: center;
  align-items: center;
}

.settings-window {
  width: 700px;
  height: 500px;
  background: rgba(30, 30, 30, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.6);
  border-radius: 12px;
  display: flex;
  overflow: hidden;
  animation: slideUp 0.3s ease-out;
}

@keyframes slideUp {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
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
  background: rgba(247, 206, 70, 0.1); /* Yellow tint */
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
</style>
