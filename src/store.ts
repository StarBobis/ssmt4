import { reactive, watch } from 'vue'
import { invoke } from '@tauri-apps/api/core'

// Define the shape of our settings
export interface AppSettings {
  bgType: 'image' | 'video';
  bgImage: string;
  bgVideo: string;
  sidebarOpacity: number;
  sidebarBlur: number;
  contentOpacity: number;
  contentBlur: number;
}

const defaultSettings: AppSettings = {
  bgType: 'image',
  bgImage: '/background.png',
  bgVideo: '/background.webm',
  sidebarOpacity: 0.3, // Lower default for dark theme transparency
  sidebarBlur: 20,
  contentOpacity: 0.2, // Lower default for dark theme transparency
  contentBlur: 3
}

export const appSettings = reactive<AppSettings>({ ...defaultSettings })

// Load from backend
async function loadSettings() {
  try {
    const loaded = await invoke<AppSettings>('loadSettings')
    // Update reactive object with loaded values
    Object.assign(appSettings, loaded)
  } catch (e) {
    console.error('Failed to load settings:', e)
  }
}

// Initial load
loadSettings()

// Auto-save behavior
watch(appSettings, async (newVal) => {
  try {
    await invoke('saveSettings', { config: newVal })
  } catch (e) {
    console.error('Failed to save settings:', e)
  }
})
