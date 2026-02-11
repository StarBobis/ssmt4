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
}

export interface GameInfo {
  name: string;
  iconPath: string;
  bgPath: string;
  bgVideoPath?: string;
  rawIcon?: string;
  rawBg?: string;
  rawBgVideo?: string;
  showSidebar: boolean;
}

const defaultSettings: AppSettings = {
  bgType: 'image',
  bgImage: '/background.png',
  bgVideo: '/background.webm',
  sidebarOpacity: 0.3, // Lower default for dark theme transparency
  sidebarBlur: 20,
  contentOpacity: 0.2, // Lower default for dark theme transparency
  contentBlur: 3,
  cacheDir: '',
  currentConfigName: 'Default'
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
      
      const icon = rawIcon ? convertFileSrc(rawIcon) : '';
      const bg = rawBg ? convertFileSrc(rawBg) : '';
      const bgVideo = rawBgVideo ? convertFileSrc(rawBgVideo) : '';
      
      console.log('[store] game paths', g.name, { rawIcon, icon, rawBg, bg, rawBgVideo });
      return {
        name: g.name,
        iconPath: icon,
        bgPath: bg,
        bgVideoPath: bgVideo,
        rawIcon: rawIcon,
        rawBg: rawBg,
        rawBgVideo: rawBgVideo,
        showSidebar: (g as any).showSidebar,
      } as GameInfo;
    });
        
        gamesList.splice(0, gamesList.length, ...processed);
    } catch (e) {
        console.error('Failed to scan games:', e);
    }
}

export function switchToGame(game: GameInfo) {
    appSettings.currentConfigName = game.name;

    // Check if we should stay in video mode
    if (appSettings.bgType === 'video') {
        // Only switch if the game has a valid video background
        if (game.rawBgVideo) {
            appSettings.bgVideo = convertFileSrc(game.rawBgVideo);
            // Explicitly stay in video mode (though already is)
            appSettings.bgType = 'video';
        } else {
             // Do nothing means keep previous video.
             // "Don't switch to corresponding background image unless..."
             console.log(`[Store] Keeping previous video for ${game.name} because it has no video bg.`);
        }
    } else {
        // Not in video mode, use standard behavior (switch to image)
        appSettings.bgType = 'image';
        // Prefer storing raw filesystem path in settings (avoid saving converted asset URL)
        // If rawBg is available use it; otherwise fall back to bgPath
        // @ts-ignore
        const rawBg = (game as any).rawBg || '';
        // convert raw fs path to asset url if present, otherwise reuse pre-converted bgPath
        appSettings.bgImage = rawBg ? convertFileSrc(rawBg) : (game.bgPath || '');
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
