import { reactive, watch, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'

// Global UI State
export const isDrawerOpen = ref(false);

export enum BGType {
  Image = 'Image',
  Video = 'Video'
}

export type Locale = 'en' | 'zhs' | 'zht';

// Define the shape of our settings
export interface AppSettings {
  bgType: BGType;
  bgImage: string;
  bgVideo: string;
  sidebarOpacity: number;
  sidebarBlur: number;
  contentOpacity: number;
  contentBlur: number;
  cacheDir: string;
  currentConfigName: string;
  githubToken: string;
  showMods: boolean;
  showWorkbench: boolean;
  showStickers: boolean;
  showWebsites: boolean;
  showDocuments: boolean;
  locale: Locale;  // 我新增
}

export interface GameInfo {
  name: string;
  iconPath: string;
  bgPath: string;
  bgVideoPath?: string;
  bgType: BGType;
  showSidebar: boolean;
}

const defaultSettings: AppSettings = {
  bgType: BGType.Image,
  bgImage: '',
  bgVideo: '',
  sidebarOpacity: 0.3,
  sidebarBlur: 20,
  contentOpacity: 0,
  contentBlur: 0,
  cacheDir: '',
  currentConfigName: 'Default',
  githubToken: '',
  showMods: true,
  showWorkbench: false,
  showStickers: false,
  showWebsites: false,
  showDocuments: false,
  locale: 'en' // 新增
}

export const appSettings = reactive<AppSettings>({ ...defaultSettings })
export const gamesList = reactive<GameInfo[]>([])

// Initial load
let isInitialized = false;

async function loadSettings() {
  try {
    const loaded = await invoke<AppSettings>('load_settings')
    console.log('Loaded settings from backend:', loaded);
    Object.assign(appSettings, loaded)
    setTimeout(() => {
      isInitialized = true;
    }, 100);
  } catch (e) {
    console.error('Failed to load settings:', e)
    await message(`加载设置失败: ${e}`, { title: '错误', kind: 'error' });
  }
}


// Default background path
let defaultBgPath = '';

async function initDefaultBackground() {
    try {
        const path = await invoke<string>('get_resource_path', { filename: 'Background.png' });
        defaultBgPath = convertFileSrc(path);
        // If current bgImage is empty, apply it immediately
        if (!appSettings.bgImage && appSettings.bgType === BGType.Image) {
            appSettings.bgImage = defaultBgPath;
        }
    } catch (e) {
        console.warn('Failed to get default background:', e);
    }
}

export async function loadGames() {
  try {
    const games = await invoke<GameInfo[]>('scan_games');
    console.log('Scanned games:', games);

    // 简化转换逻辑，直接使用后端返回的字段
    const processed = games.map(g => {
      const timestamp = Date.now();

      return {
        name: g.name,
        // 直接使用后端返回的 camelCase 字段
        iconPath: g.iconPath ? convertFileSrc(g.iconPath) + `?t=${timestamp}` : '',
        bgPath: g.bgPath ? convertFileSrc(g.bgPath) + `?t=${timestamp}` : '',
        bgVideoPath: g.bgVideoPath ? convertFileSrc(g.bgVideoPath) + `?t=${timestamp}` : undefined,
        bgType: g.bgType || BGType.Image,
        showSidebar: g.showSidebar,
      } as GameInfo;
    });

    gamesList.splice(0, gamesList.length, ...processed);
    
    // Ensure default background is loaded
    await initDefaultBackground();

    // Refresh current game background if it exists
    if (appSettings.currentConfigName) {
      const current = gamesList.find(g => g.name === appSettings.currentConfigName);
      if (current) {
        switchToGame(current);
      }
    } else {
        // If no game selected, ensure default background is shown
         if (!appSettings.bgImage && appSettings.bgType === BGType.Image) {
            appSettings.bgImage = defaultBgPath;
        }
    }
  } catch (e) {
    console.error('Failed to scan games:', e);
  }
}

export function switchToGame(game: GameInfo) {
  appSettings.currentConfigName = game.name;

  // 使用 game.bgType 来决定显示类型
  const useVideo = game.bgType === BGType.Video;

  if (useVideo && game.bgVideoPath) {
    appSettings.bgType = BGType.Video;
    appSettings.bgVideo = game.bgVideoPath;
  } else {
    appSettings.bgType = BGType.Image;
    appSettings.bgImage = game.bgPath || defaultBgPath;
  }
}

// Initial load
loadSettings();
loadGames();

// Auto-save behavior
watch(appSettings, async (newVal) => {
  if (!isInitialized) {
    console.log('Skipping save because store is not yet initialized');
    return;
  }
  console.log('Saving settings:', newVal);
  try {
    await invoke('save_settings', { config: newVal })
  } catch (e) {
    console.error('Failed to save settings:', e)
  }
}, { deep: true })