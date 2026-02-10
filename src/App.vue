<script setup lang="ts">
import { ref, computed } from "vue";
import { appSettings } from "./store";
import TitleBar from "./components/TitleBar.vue";

const isCollapse = ref(false);

const bgStyle = computed(() => {
  if (appSettings.bgType === 'image') {
    return {
      backgroundImage: `url(${appSettings.bgImage})`,
      backgroundSize: 'cover',
      backgroundPosition: 'center',
    }
  }
  return {}
})
</script>

<template>
  <!-- Custom Title Bar -->
  <TitleBar>
  </TitleBar>

  <!-- Background Layer -->
  <div class="bg-layer" :style="bgStyle">
    <video 
      v-if="appSettings.bgType === 'video'" 
      :src="appSettings.bgVideo" 
      autoplay loop muted playsinline 
      class="bg-video"
    ></video>
  </div>

  <el-config-provider>
    <el-container class="app-container">
      <el-aside width="auto" class="app-aside" :style="{
        '--sidebar-bg-opacity': appSettings.sidebarOpacity,
        '--sidebar-blur': `${appSettings.sidebarBlur}px`
      }">
        <div class="aside-flex">
          
          <el-menu
            :default-active="$route.path"
            class="menu-top"
            router
            :collapse="isCollapse"
          >
            <el-menu-item index="/">
              <el-icon><HomeFilled /></el-icon>
              <span>主页</span>
            </el-menu-item>
            <el-menu-item index="/workbench">
              <el-icon><Monitor /></el-icon>
              <span>工作台</span>
            </el-menu-item>
            <el-menu-item index="/stickers">
              <el-icon><Picture /></el-icon>
              <span>贴图标记</span>
            </el-menu-item>
            <el-menu-item index="/websites">
              <el-icon><Link /></el-icon>
              <span>常用网址</span>
            </el-menu-item>
          </el-menu>

          <el-menu
            :default-active="$route.path"
            class="menu-bottom"
            router
            :collapse="isCollapse"
          >
            <el-menu-item index="/settings">
              <el-icon><Setting /></el-icon>
              <span>设置</span>
            </el-menu-item>
          </el-menu>
        </div>
      </el-aside>
      
      <el-main class="app-main" :style="{
        '--content-bg-opacity': appSettings.contentOpacity,
        '--content-blur': `${appSettings.contentBlur}px`
      }">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </el-main>
    </el-container>
  </el-config-provider>
</template>

<style>
/* Global Resets */
html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  background-color: #000; /* Dark bg */
}
#app {
  height: 100%;
  position: relative; /* Need relative for absolute children */
}
.bg-layer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0;
  overflow: hidden;
}
.bg-video {
  width: 100%;
  height: 100%;
  object-fit: cover;
}
</style>

<style scoped>
.app-container {
  height: 100vh;
  overflow: hidden;
  background-color: transparent;
  position: relative;
  z-index: 1; /* Above bg */
}

.app-aside {
  /* Glassmorphism effect - Configurable */
  /* Switched to Black base (rgba(0,0,0)) to ensure white text remains visible even at 100% opacity */
  background-color: rgba(0, 0, 0, var(--sidebar-bg-opacity, 0.3)) !important; 
  backdrop-filter: blur(var(--sidebar-blur, 20px)) saturate(150%);
  -webkit-backdrop-filter: blur(var(--sidebar-blur, 20px)) saturate(150%);
  border-right: 1px solid rgba(255, 255, 255, 0.15); /* Subtler border for dark theme */
  box-shadow: 2px 0 12px rgba(0, 0, 0, 0.2); 
  transition: background-color 0.3s, backdrop-filter 0.3s;
}

.aside-flex {
  display: flex;
  flex-direction: column;
  height: 100%;
  background-color: transparent;
}

/* Menu Visibility Tuning */
:deep(.el-menu) {
  background-color: transparent !important;
  border-right: none;
}

/* Default State: White text with shadow for readability on any background */
:deep(.el-menu-item), :deep(.el-sub-menu__title) {
  background-color: transparent !important;
  color: #ffffff !important;
  text-shadow: 0 1px 4px rgba(0, 0, 0, 0.8); /* Strong shadow */
  transition: background-color 0.2s, color 0.2s;
}

/* Icons: Same treatment */
:deep(.el-menu-item .el-icon), :deep(.el-sub-menu__title .el-icon) {
  color: #ffffff !important;
  filter: drop-shadow(0 1px 2px rgba(0, 0, 0, 0.6));
}

/* Hover State: Subtle light wash */
:deep(.el-menu-item:hover), :deep(.el-sub-menu__title:hover) {
  background-color: rgba(255, 255, 255, 0.2) !important;
}

/* Active State: Glassmorphism style */
:deep(.el-menu-item.is-active) {
  background-color: rgba(255, 255, 255, 0.25) !important;
  border-right: 3px solid #ffffff; /* Use a border to mark active */
  font-weight: 600;
  /* Keep text shadow for readability */
}

.menu-top {
  flex-grow: 1;
  border-right: none !important;
  overflow-y: auto;
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE/Edge */
}
.menu-top::-webkit-scrollbar {
  display: none; /* Chrome/Safari */
}

.menu-bottom {
  flex-grow: 0;
  border-right: none !important;
  border-top: 1px solid rgba(0,0,0,0.1);
}

.app-main {
  padding: 52px 20px 20px 20px;
  /* Content area: Configurable */
  background-color: rgba(255, 255, 255, var(--content-bg-opacity, 0.55)); 
  backdrop-filter: blur(var(--content-blur, 3px)); 
  overflow-y: auto;
  overflow-y: auto;
  transition: opacity 0.5s ease;
  
  /* Dark Glass Style Overrides */
  background-color: rgba(0, 0, 0, var(--content-bg-opacity, 0.4)); 
  color: #ffffff;
}

/* Glassmorphism for Element Plus Components */
:deep(.el-card) {
  background-color: rgba(30, 30, 30, 0.6) !important;
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #ffffff;
  --el-card-bg-color: transparent;
}
:deep(.el-card__header) {
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  color: #fff;
}
/* Form Labels */
:deep(.el-form-item__label) {
  color: #e0e0e0 !important;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>