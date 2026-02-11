<script setup lang="ts">
import { computed, onMounted, onUnmounted } from "vue";
import { useRoute } from "vue-router";
import { appSettings } from "./store";
import TitleBar from "./components/TitleBar.vue";

const route = useRoute();

// Disable default right-click context menu
const preventContextMenu = (event: Event) => {
  event.preventDefault();
};

onMounted(() => {
  document.addEventListener('contextmenu', preventContextMenu);
});

onUnmounted(() => {
  document.removeEventListener('contextmenu', preventContextMenu);
});

/* bgStyle removed, handled in template */
</script>

<template>
  <!-- Custom Title Bar -->
  <TitleBar>
  </TitleBar>

  <!-- Background Layer -->
  <div class="bg-layer">
    <transition-group name="bg-trans">
      <!-- Image Background -->
      <div 
        v-if="appSettings.bgType === 'image' && appSettings.bgImage"
        :key="appSettings.bgImage"
        class="bg-item"
        :style="{ backgroundImage: `url(${appSettings.bgImage})` }"
      ></div>

      <!-- Video Background -->
      <video 
        v-if="appSettings.bgType === 'video' && appSettings.bgVideo" 
        :key="appSettings.bgVideo"
        :src="appSettings.bgVideo" 
        autoplay loop muted playsinline 
        class="bg-item"
      ></video>
    </transition-group>
  </div>
  
  <!-- Home Ambient Shadow Layer -->
  <div class="home-shadow-layer" v-if="route.path === '/'"></div>

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
        <div class="content-scroll-wrapper" :class="{ 'no-scroll': route.path === '/' }">
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
  
  /* Disable text selection */
  user-select: none;

  /* Cyberpunk Black Fallback: Deep dark with subtle neon glows */
  background-color: #030305;
  background-image: 
    radial-gradient(circle at 50% 50%, rgba(60, 20, 100, 0.2) 0%, transparent 60%),
    radial-gradient(circle at 50% 50%, rgba(0, 100, 180, 0.1) 0%, transparent 70%);

  overflow: hidden;
}

/* Re-enable selection for inputs */
input, textarea {
  user-select: text;
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
  background-color: #050505; /* Black fallback for transitions */
}

/* Background Transition Items */
.bg-item {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  background-size: cover;
  background-position: center;
  will-change: opacity;
}

/* Transition Classes */
.bg-trans-enter-active,
.bg-trans-leave-active {
  transition: opacity 0.6s ease; /* Smooth 0.6s fade */
}

.bg-trans-enter-from,
.bg-trans-leave-to {
  opacity: 0;
}

.bg-video {
  /* Removed, replaced by .bg-item */
}

.home-shadow-layer {
  position: fixed;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  z-index: 0; /* On top of bg-layer (also 0, but later in DOM), behind app-container (1) */
  pointer-events: none;
  background: 
    /* 1. Bottom-Left Shadow: subtle grounding */
    radial-gradient(circle at 0% 100%, rgba(0, 0, 0, 0.4) 0%, transparent 50%),
    
    /* 2. Bottom-Right Shadow: subtle grounding */
    radial-gradient(circle at 100% 100%, rgba(0, 0, 0, 0.4) 0%, transparent 50%),
    
    /* 3. Top-Right to Center-ish Shadow: dramatic diagonal shading */
    linear-gradient(225deg, rgba(0, 0, 0, 0.3) 0%, transparent 60%);
  mix-blend-mode: multiply;
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
  padding: 0 0 32px 0; /* Add 32px bottom padding globally */
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

.no-scroll {
  overflow-y: hidden !important;
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