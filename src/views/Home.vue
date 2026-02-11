<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
// import  nextTick from 'vue'
import { gamesList, switchToGame, appSettings, loadGames } from '../store' 
import { invoke } from '@tauri-apps/api/core'
// import { openPath } from '@tauri-apps/plugin-opener'; // Import openPath
import { join } from '@tauri-apps/api/path';
import GameSettingsModal from '../components/GameSettingsModal.vue'

// Computed property to get sidebar games (filtered and reverse order)
const sidebarGames = computed(() => {
  return gamesList
    .filter(g => g.showSidebar)
    .reverse(); 
});

const isGameActive = (gameName: string) => {
    return appSettings.currentConfigName === gameName;
};

const handleGameClick = (game: any) => {
    switchToGame(game);
}

// Context Menu State
const showMenu = ref(false);
const menuX = ref(0);
const menuY = ref(0);
const targetGame = ref<any>(null);

const handleContextMenu = (e: MouseEvent, game: any) => {
  e.preventDefault();
  targetGame.value = game;
  menuX.value = e.clientX;
  menuY.value = e.clientY;
  showMenu.value = true;
};

const closeMenu = () => {
  showMenu.value = false;
};

const hideGame = async () => {
  if (!targetGame.value) return;
  
  const gameName = targetGame.value.name;
  const wasActive = isGameActive(gameName);
  
  try {
    await invoke('set_game_visibility', { gameName, visible: false });
    await loadGames();
    
    // If the hidden game was active, switch to the first available game
    if (wasActive && sidebarGames.value.length > 0) {
      switchToGame(sidebarGames.value[0]);
    }
  } catch (err) {
    console.error('Failed to hide game:', err);
  }
  
  closeMenu();
};

const open3dmigotoFolder = async () => {
    const gameName = appSettings.currentConfigName;
    if (!gameName || gameName === 'Default') return;

    try {
        // We need to load the config to find the path
        const data = await invoke<any>('load_game_config', { gameName });
        let path = data.threeDMigoto?.installDir;
        
        // Default Logic (Must match Modal logic)
        if (!path && appSettings.cacheDir) {
            path = await join(appSettings.cacheDir, '3Dmigoto', gameName);
        }

        if (path) {
            await invoke('ensure_directory', { path });
            await invoke('open_in_explorer', { path });
        } else {
            console.warn('No 3Dmigoto path found and no cache dir set.');
        }
    } catch (e) {
        console.error('Failed to open 3Dmigoto folder:', e);
    }
};

const openD3dxIni = async () => {
    const gameName = appSettings.currentConfigName;
    if (!gameName || gameName === 'Default') return;

    try {
        let path: string | undefined;
        // Load config to find path
        const data = await invoke<any>('load_game_config', { gameName });
        path = data.threeDMigoto?.installDir;
        
        // Fallback
        if (!path && appSettings.cacheDir) {
            path = await join(appSettings.cacheDir, '3Dmigoto', gameName);
        }

        if (path) {
            await invoke('ensure_directory', { path });
            const iniPath = await join(path, 'd3dx.ini');
            await invoke('open_in_explorer', { path: iniPath });
        }
    } catch (e) {
        console.error('Failed to open d3dx.ini:', e);
    }
};

const showSettings = ref(false);

onMounted(() => {
  document.addEventListener('click', closeMenu);
});

onUnmounted(() => {
  document.removeEventListener('click', closeMenu);
});
</script>

<template>
  <div class="home-container">
    <div class="sidebar-wrapper">
        <div class="sidebar-track">
          <!-- Games Loop -->
           <el-tooltip
            v-for="game in sidebarGames"
            :key="game.name"
            :content="game.name"
            placement="right"
            effect="dark"
            popper-class="game-tooltip"
          >
            <div 
                class="sidebar-icon" 
                :class="{ active: isGameActive(game.name) }"
                @click.stop="handleGameClick(game)"
                @contextmenu.prevent="handleContextMenu($event, game)"
            >
              <img 
                  :src="game.iconPath" 
                  :alt="game.name" 
                  loading="lazy" 
                  @load="(e) => (e.target as HTMLImageElement).style.opacity = '1'"
                  @error="(e) => (e.target as HTMLImageElement).style.opacity = '0'"
              />
            </div>
          </el-tooltip>
        </div>
    </div>

    <!-- Custom Context Menu -->
    <div 
      v-if="showMenu" 
      class="context-menu" 
      :style="{ top: menuY + 'px', left: menuX + 'px' }"
      @click.stop
    >
      <div class="menu-item" @click="hideGame">
        不显示此游戏
      </div>
    </div>

    <div class="content-area">



    </div>

    <!-- Settings Modal -->
    <GameSettingsModal 
      v-model="showSettings" 
      :game-name="appSettings.currentConfigName"
    />

    <div class="action-bar">
      <!-- Start Game Button -->
      <div class="start-game-btn">
        <div class="icon-wrapper">
          <div class="play-triangle"></div>
        </div>
        <span class="btn-text">开始游戏</span>
      </div>

      <!-- Settings Menu Button -->
      <el-dropdown trigger="hover" placement="top-end" popper-class="settings-dropdown">
        <div class="settings-btn">
          <div class="menu-lines">
             <div class="line"></div>
             <div class="line"></div>
             <div class="line"></div>
          </div>
        </div>
        <template #dropdown>
          <el-dropdown-menu>
            <el-dropdown-item @click="showSettings = true">游戏设置</el-dropdown-item>
            <el-dropdown-item @click="open3dmigotoFolder">打开3Dmigoto文件夹</el-dropdown-item>
            <el-dropdown-item @click="openD3dxIni">打开d3dx.ini</el-dropdown-item>
            <el-dropdown-item divided>开启Symlink</el-dropdown-item>
            <el-dropdown-item>关闭Symlink</el-dropdown-item>
            <el-dropdown-item divided>检查3Dmigoto包更新</el-dropdown-item>

          </el-dropdown-menu>
        </template>
      </el-dropdown>
    </div>
      




  </div>
</template>

<style scoped>
.home-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: row; /* Changed to row for Sidebar + Content */
  padding: 0; /* Remove padding from container, move to children */
  box-sizing: border-box;
  position: relative;
}

.sidebar-wrapper {
    width: 80px;
    height: 100%;
    /* Gradient Background: Transparent top to Black bottom */
    background: linear-gradient(to bottom, rgba(0, 0, 0, 0) 0%, rgba(0, 0, 0, 0.95) 100%);
    display: flex;
    flex-direction: column;
    /* justify-content: flex-end; Removed to allow scrolling with margin-top: auto */
    padding-bottom: 16px; /* Space from bottom matches side margins */
    padding-top: 40px; /* Safe area for TitleBar when scrolling */
    box-sizing: border-box;
    z-index: 10;
    
    overflow-y: auto;
    overflow-x: hidden;
    
    /* Distinct right border */
    border-right: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 2px 0 10px rgba(0, 0, 0, 0.5); /* Adds depth shadow to the right */
}

/* Hide scrollbar for sidebar */
.sidebar-wrapper::-webkit-scrollbar {
  width: 0px;
  background: transparent;
}

.sidebar-track {
    display: flex;
    flex-direction: column-reverse; /* Stack from bottom to top as requested */
    gap: 16px;
    align-items: center;
    width: 100%;
    margin-top: auto; /* Push content to bottom when not overflowing */
}

.sidebar-icon {
    width: 48px;
    height: 48px;
    border-radius: 12px;
    overflow: hidden;
    cursor: pointer;
    transition: transform 0.2s ease, box-shadow 0.2s ease;
    background-color: rgba(0,0,0,0.3); /* Placeholder bg */
}

.sidebar-icon img {
    width: 100%;
    height: 100%;
    object-fit: cover;
}

.sidebar-icon:hover {
    transform: scale(1.1);
    box-shadow: 0 0 10px rgba(255, 255, 255, 0.3);
}

/* Crystal/Amber selection effect */
.sidebar-icon.active {
    /* 
       Layered Box Shadows to create "Gap + Thick Border" effect
       1. Dark gap (simulating distance from icon)
       2. Thick White Border
       3. Outer Glow
       4. Inner Glow (Crystal effect)
    */
    box-shadow: 
        0 0 0 2px rgba(0, 0, 0, 0.6),  /* 2px Distance/Gap */
        0 0 0 4px #ffffff,            /* 2px Thick White Border (4px total spread - 2px gap) */
        0 0 20px rgba(255, 255, 255, 0.5), /* Soft ambient glow */
        inset 0 0 20px rgba(255, 255, 255, 0.5); /* Inner crystal glow */
        
    /* Remove physical border or make it subtle inner edge */
    border: 1px solid rgba(255, 255, 255, 0.3);
    
    transform: scale(1.05);
    z-index: 2; /* Ensure shadow overlaps neighbors if needed */
}

.content-area {
  flex: 1;
  display: flex; 
  flex-direction: column;
  padding: 40px; /* Restore padding here */
  position: relative; 
  z-index: 1; /* Ensure content sits above shadow */
}

.action-bar {
  display: flex;
  height: 60px; /* Tall button strip */
  /* Remove flex-end self align because now it is inside content-area which needs to be carefully managed */
  /* Or actually, keep it but ensure content-area is full height */
  margin-top: auto; /* Push to bottom */
  align-self: flex-end;
  gap: 16px; /* Space between buttons */
  
  /* Ensure it doesn't overlap with sidebar if window is small, though structure prevents it */
  padding-right: 40px; /* Right padding from screen edge */
  padding-bottom: 40px;
}

/* --- Start Game Button --- */
.start-game-btn {
  background-color: #F7CE46; /* Yellow */
  color: #000000;
  display: flex;
  align-items: center;
  padding: 0 24px 0 12px; /* Adjusted padding-right to balance with text margin */
  cursor: pointer;
  transition: all 0.2s ease;
  font-family: 'Microsoft YaHei', sans-serif;
  
  /* Full rounded capsule shape */
  border-radius: 30px;
}

.start-game-btn .btn-text {
  font-size: 20px;
  font-weight: 900;
  margin-left: 16px;
  letter-spacing: 2px;
}

.icon-wrapper {
  width: 36px;
  height: 36px;
  background-color: #000000;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
}

.play-triangle {
  width: 0;
  height: 0;
  border-style: solid;
  border-width: 7px 0 7px 11px; /* Pointing right */
  border-color: transparent transparent transparent #F7CE46; /* Yellow triangle */
  margin-left: 3px; /* Visual optical adjustment */
  transition: all 0.2s ease;
}

/* Hover Effect: Flip Colors for Start Button */
.start-game-btn:hover {
  background-color: #333333;
  color: #F7CE46;
}

.start-game-btn:active {
  background-color: #000000;
}

.start-game-btn:hover .icon-wrapper {
  background-color: #F7CE46;
}

.start-game-btn:hover .play-triangle {
  border-color: transparent transparent transparent #333333;
}

.start-game-btn:active .play-triangle {
  border-color: transparent transparent transparent #000000;
}


/* --- Settings Button --- */
/* Wrapper for dropdown to behave as flex item */
:deep(.el-dropdown) {
  display: flex;
  align-items: stretch;
}

.settings-btn {
  width: 60px; /* Square to make it a circle (height is 60px from parent) */
  background-color: #2D2D2D; /* Dark Gray */
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  
  /* Remove default borders/outlines */
  border: none;
  outline: none;
  
  /* Circle shape */
  border-radius: 50%;
  
  transition: background-color 0.2s;
}

/* Ensure no outline on focus */
.settings-btn:focus,
.settings-btn:focus-visible {
  outline: none;
  border: none;
}

.settings-btn:hover {
  background-color: #2D2D2D; /* Keep background unchanged on hover as requested, or slightly lighter? User said "bg color is gray keep unchanged". Assuming static. */
}

/* The three horizontal lines icon */
.menu-lines {
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  height: 16px;
  width: 20px;
}

.line {
  height: 3px;
  background-color: #888888; /* Gray lines */
  width: 100%;
  border-radius: 2px;
  transition: background-color 0.2s;
}

.settings-btn:hover .line {
  background-color: #ffffff; /* White lines on hover */
}

/* Context Menu */
.context-menu {
  position: fixed;
  z-index: 10000;
  background: rgba(30, 30, 30, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(8px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.5);
  border-radius: 6px;
  padding: 4px;
  min-width: 120px;
}

.menu-item {
  padding: 8px 12px;
  cursor: pointer;
  color: #eee;
  font-size: 13px;
  border-radius: 4px;
  transition: background-color 0.1s;
}

.menu-item:hover {
  background-color: rgba(255, 255, 255, 0.1);
  color: #fff;
}
</style>

<style>
/* Global styles for the settings dropdown */
.settings-dropdown.el-popper {
  border-radius: 12px !important;
  overflow: hidden;
}

.settings-dropdown .el-dropdown-menu {
  border-radius: 12px !important;
  padding: 6px !important;
}

.settings-dropdown .el-dropdown-menu__item {
  border-radius: 8px !important;
  margin-bottom: 2px;
}
.settings-dropdown .el-dropdown-menu__item:last-child {
  margin-bottom: 0;
}
</style>
