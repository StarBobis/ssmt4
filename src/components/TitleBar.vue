<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';

const appWindow = getCurrentWindow();
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
</script>

<template>
  <div class="titlebar">
    <div class="drag-region" @mousedown="startDrag">
      <div class="title-content">
          <slot></slot>
      </div>
    </div>
    
    <div class="window-controls">
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
  align-items: flex-start;
  position: fixed;
  top: 0;
  left: 0;
  z-index: 9999;
  user-select: none;
}

.drag-region {
  flex-grow: 1;
  height: 100%;
  background: transparent; /* Ensures the div is rendered */
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
