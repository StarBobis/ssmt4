import { reactive, watch, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { message } from '@tauri-apps/plugin-dialog'
import { convertFileSrc } from '@tauri-apps/api/core'

// Global UI State
export const isDrawerOpen = ref(false);

// Define the shape of our settings
export interface AppSettings {
  bgType: 'image' | 'video';
  bgImage: string;
  bgVideo: string;
  sidebarOpacity: number;
  sidebarBlur: number;
  contentOpacity: number;
  contentBlur: number;
  cacheDir: string;
  currentConfigName: string;
  githubToken: string;
}

export interface GameInfo {
  name: string;
  iconPath: string;
  bgPath: string;
  bgVideoPath?: string;
  bgType?: 'image' | 'video'; // Add this
  rawIcon?: string;
  rawBg?: string;  
  rawBgVideo?: string;
  showSidebar: boolean;
}

const defaultSettings: AppSettings = {
  bgType: 'image',
  bgImage: '',
  bgVideo: '',
  sidebarOpacity: 0.3, // Lower default for dark theme transparency
  sidebarBlur: 20,
  contentOpacity: 0.2, // Lower default for dark theme transparency
  contentBlur: 3,
  cacheDir: '',
  currentConfigName: 'Default',
  githubToken: ''
}

export const appSettings = reactive<AppSettings>({ ...defaultSettings })
export const gamesList = reactive<GameInfo[]>([])


// Initial load
let isInitialized = false;

async function loadSettings() {
  try {
    const loaded = await invoke<AppSettings>('load_settings')
    console.log('Loaded settings from backend:', loaded);
    
    // Update reactive object with loaded values
    Object.assign(appSettings, loaded)
    
    // Mark as initialized so watch can start saving changes
    // using setTimeout to ensure the current tick doesn't trigger watch
    setTimeout(() => {
        isInitialized = true;
    }, 100);
    
  } catch (e) {
    console.error('Failed to load settings:', e)
    await message(`加载设置失败: ${e}`, { title: '错误', kind: 'error' });
  }
}


export async function loadGames() {
    try {
        const games = await invoke<GameInfo[]>('scan_games');
        console.log('Scanned games:', games);
        
        // Transform paths for frontend usage
    const processed = games.map(g => {
      const rawIcon = (g as any).icon_path || (g as any).iconPath || '';
      const rawBg = (g as any).bg_path || (g as any).bgPath || '';
      const rawBgVideo = (g as any).bgVideoPath || (g as any).bg_video_path || '';
      
      const timestamp = Date.now();
      const icon = rawIcon ? convertFileSrc(rawIcon) + `?t=${timestamp}` : '';
      const bg = rawBg ? convertFileSrc(rawBg) + `?t=${timestamp}` : '';
      const bgVideo = rawBgVideo ? convertFileSrc(rawBgVideo) + `?t=${timestamp}` : '';
      
      console.log('[store] game paths', g.name, { rawIcon, icon, rawBg, bg, rawBgVideo });
      return {
        name: g.name,
        iconPath: icon,
        bgPath: bg,
        bgVideoPath: bgVideo,
        bgType: (g as any).bgType || 'image',
        rawIcon: rawIcon,
        rawBg: rawBg,
        rawBgVideo: rawBgVideo,
        showSidebar: (g as any).showSidebar,
      } as GameInfo;
    });
        
    gamesList.splice(0, gamesList.length, ...processed);
    
    // Refresh current game background if it exists (force update timestamps)
    if (appSettings.currentConfigName) {
        const current = gamesList.find(g => g.name === appSettings.currentConfigName);
        if (current) {
            switchToGame(current);
        }
    }
    } catch (e) {
        console.error('Failed to scan games:', e);
    }
}

export function switchToGame(game: GameInfo) {
    appSettings.currentConfigName = game.name;

    // Determine type based on game config
    const useVideo = game.bgType === 'video';
    
    if (useVideo && game.bgVideoPath) {
         appSettings.bgType = 'video';
         appSettings.bgVideo = game.bgVideoPath;
    } else {
         appSettings.bgType = 'image';
         appSettings.bgImage = game.bgPath || '';
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
