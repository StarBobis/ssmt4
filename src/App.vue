<script setup lang="ts">
import { computed } from "vue";
import { useRoute } from "vue-router";
import { appSettings } from "./store";
import TitleBar from "./components/TitleBar.vue";

const route = useRoute();

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
  
  <!-- Global Mask Layer for Game Library Page -->
  <transition name="fade">
    <div v-if="route.path === '/games'" class="global-dim-layer"></div>
  </transition>

  <el-config-provider>
    <div class="app-container">
      <main class="app-main" :style="{
        '--content-bg-opacity': appSettings.contentOpacity,
        '--content-blur': `${appSettings.contentBlur}px`
      }">
        <div class="content-scroll-wrapper">
          <router-view v-slot="{ Component }">
            <transition name="fade" mode="out-in">
              <component :is="Component" />
            </transition>
          </router-view>
        </div>
      </main>
    </div>
  </el-config-provider>
</template>

<style>
/* Global Resets */
html, body {
  margin: 0;
  padding: 0;
  height: 100%;
  font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
  
  /* Cyberpunk Black Fallback: Deep dark with subtle neon glows */
  background-color: #030305;
  background-image: 
    radial-gradient(circle at 50% 50%, rgba(60, 20, 100, 0.2) 0%, transparent 60%),
    radial-gradient(circle at 50% 50%, rgba(0, 100, 180, 0.1) 0%, transparent 70%);

  overflow: hidden;
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

.global-dim-layer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0; /* Above bg-layer (0 via DOM order), below App Content (1) */
  pointer-events: none; /* Let clicks pass through if needed, though standard bg doesn't need interactions */
  
  /* Center Radiating Light Background (Copied from GameLibrary.vue) */
    background: radial-gradient(
        circle at 50% 50%, 
        rgba(0, 0, 0, 0.6) 0%, 
        rgba(0, 0, 0, 0.9) 50%, 
        rgba(0, 0, 0, 0.98) 90%
    );
}
</style>

<style scoped>
.app-container {
  height: 100vh;
  width: 100vw;
  overflow: hidden;
  position: relative;
  z-index: 1; /* Above bg */
  padding-top: 32px; /* TitleBar height */
}

.app-main {
  width: 100%;
  height: 100%;
  padding: 0;
  overflow: hidden;
  position: relative;
  /* Content area: Configurable */
  background-color: rgba(255, 255, 255, var(--content-bg-opacity, 0.55)); 
  backdrop-filter: blur(var(--content-blur, 3px)); 
  transition: opacity 0.5s ease;
  
  /* Dark Glass Style Overrides */
  background-color: rgba(0, 0, 0, var(--content-bg-opacity, 0.4)); 
  color: #ffffff;
}

.content-scroll-wrapper {
  margin-top: 0;
  height: 100%;
  overflow-y: auto;
  padding: 24px; /* Added spacing to prevent content from touching edges */
  box-sizing: border-box; /* Ensures padding doesn't cause overflow */
}

/* Custom Scrollbar for Content */
.content-scroll-wrapper::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
.content-scroll-wrapper::-webkit-scrollbar-track {
  background: transparent;
}
.content-scroll-wrapper::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.2); 
  border-radius: 4px;
}
.content-scroll-wrapper::-webkit-scrollbar-thumb:hover {
  background: rgba(255, 255, 255, 0.4); 
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