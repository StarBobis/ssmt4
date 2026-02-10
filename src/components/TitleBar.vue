<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useRouter, useRoute } from 'vue-router';

const appWindow = getCurrentWindow();
const router = useRouter();
const route = useRoute();
const isMaximized = ref(false);

const checkMaximized = async () => {
    isMaximized.value = await appWindow.isMaximized();
};

onMounted(async () => {
    checkMaximized();
    // Listen to resize event to update maximized state icon
    const unlisten = await appWindow.onResized(() => {
        checkMaximized();
    });
    
    // Setup cleanup
    onUnmounted(() => {
        unlisten();
    });
});

const minimize = () => appWindow.minimize();
const toggleMaximize = async () => {
    await appWindow.toggleMaximize();
    checkMaximized();
};
const close = () => appWindow.close();
const startDrag = () => {
  appWindow.startDragging();
};

const toggleGamePage = (e: MouseEvent) => {
    e.stopPropagation(); // prevent drag
    if (route.path === '/games') {
        router.push('/');
    } else {
        router.push('/games');
    }
};

const navTo = (path: string) => {
    router.push(path);
};
</script>

<template>
  <div class="titlebar">
    <div class="nav-controls">
        <div class="nav-button" :class="{ active: route.path === '/' }" @click="navTo('/')" title="Home">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
            <span class="nav-text">Home</span>
        </div>
        <div class="nav-button" :class="{ active: route.path === '/workbench' }" @click="navTo('/workbench')" title="Workbench">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="2" y="3" width="20" height="14" rx="2" ry="2"></rect><line x1="8" y1="21" x2="16" y2="21"></line><line x1="12" y1="17" x2="12" y2="21"></line></svg>
            <span class="nav-text">Workbench</span>
        </div>
        <div class="nav-button" :class="{ active: route.path === '/stickers' }" @click="navTo('/stickers')" title="Stickers">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect><circle cx="8.5" cy="8.5" r="1.5"></circle><polyline points="21 15 16 10 5 21"></polyline></svg>
            <span class="nav-text">Stickers</span>
        </div>
        <div class="nav-button" :class="{ active: route.path === '/websites' }" @click="navTo('/websites')" title="Websites">
            <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>
            <span class="nav-text">Websites</span>
        </div>
    </div>

    <div class="drag-region" @mousedown="startDrag">
      <div class="title-content">
          <slot></slot>
      </div>
    </div>
    
    <div class="window-controls">
      <!-- Game List Toggle Button -->
      <div class="control-button game-list-toggle" :class="{ active: route.path === '/games' }" @click="toggleGamePage" title="Switch to Game Library">
        <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <rect x="3" y="3" width="7" height="7"></rect>
          <rect x="14" y="3" width="7" height="7"></rect>
          <rect x="14" y="14" width="7" height="7"></rect>
          <rect x="3" y="14" width="7" height="7"></rect>
        </svg>
      </div>

      <!-- Settings Button (Placed to right of Game Toggle) -->
      <div class="control-button settings-btn" :class="{ active: route.path === '/settings' }" @click="navTo('/settings')" title="Settings">
         <svg xmlns="http://www.w3.org/2000/svg" width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="3"></circle><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1.82 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"></path></svg>
      </div>

      <div class="control-button minimize" @click="minimize">
        <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
          <path d="M0,5 L10,5 L10,6 L0,6 Z" />
        </svg>
      </div>
      
      <div class="control-button maximize" @click="toggleMaximize">
        <svg v-if="!isMaximized" xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
          <path d="M1,1 L9,1 L9,9 L1,9 L1,1 Z M0,0 L0,10 L10,10 L10,0 L0,0 Z" />
        </svg>
        <svg v-else xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
           <path d="M2.1,0v2H0v8.1h8.2v-2h2V0H2.1z M7.2,9.2H1V3h6.1V9.2z M9.2,7.1h-1V2H3.1V1h6.1V7.1z" />
        </svg>
      </div>
      
      <div class="control-button close" @click="close">
        <svg xmlns="http://www.w3.org/2000/svg" width="10" height="10" viewBox="0 0 10 10">
          <path d="M1,0 L5,4 L9,0 L10,1 L6,5 L10,9 L9,10 L5,6 L1,10 L0,9 L4,5 L0,1 L1,0 Z" />
        </svg>
      </div>
    </div>
  </div>
</template>

<style scoped>
.titlebar {
  height: 32px;
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center; /* Changed to center to align items vertically */
  position: fixed;
  top: 0;
  left: 0;
  z-index: 9999;
  user-select: none;
  background: rgba(0, 0, 0, 0.2); /* Slight background for visibility */
  backdrop-filter: blur(4px);
  transition: background 0.3s ease, backdrop-filter 0.3s ease;
}

.nav-controls {
    display: flex;
    align-items: center;
    height: 100%;
    padding-left: 8px; /* Safe space from edge */
    z-index: 10001; /* Above drag region */
}

.nav-button {
    display: flex;
    align-items: center;
    padding: 0 12px;
    height: 100%;
    cursor: auto; /* It is clickable, but we set to auto to avoid global pointer. Actual clickable is fine. */
    cursor: pointer;
    font-size: 12px;
    color: rgba(255, 255, 255, 0.7);
    transition: all 0.2s;
    border-radius: 4px;
}
.nav-button:hover {
    color: #fff;
    background: rgba(255, 255, 255, 0.1);
}
.nav-button.active {
    color: #fff;
    font-weight: 600;
    background: rgba(255, 255, 255, 0.15);
}
.nav-button svg {
    margin-right: 6px;
    opacity: 0.8;
}
.nav-button.active svg {
    opacity: 1;
}

.drag-region {
  flex-grow: 1;
  height: 100%;
  background: transparent; 
}

.title-content {
    height: 100%;
    display: flex;
    align-items: center;
    font-size: 12px;
}

.window-controls {
  display: flex;
  height: 32px;
  flex-shrink: 0;
  z-index: 10001; /* Ensure buttons are top-most */
}

.control-button {
  display: flex;
  justify-content: center;
  align-items: center;
  width: 46px;
  height: 100%;
  cursor: default;
  transition: background-color 0.1s;
}

.control-button svg {
  fill: currentColor;
}

.control-button:hover {
  background-color: rgba(255, 255, 255, 0.1);
}

.control-button.close:hover {
  background-color: #e81123;
}
.control-button.close:hover svg {
    fill: white;
}
</style>
