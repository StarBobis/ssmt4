<script setup lang="ts">
// import { ref } from "vue";
// import { invoke } from "@tauri-apps/api/core";
</script>

<template>
  <el-config-provider>
    <el-container class="app-container">
      <el-aside width="250px" class="app-aside">
        <div class="aside-flex">
          <div class="app-logo">
            <h2>SSMT4 Tools</h2>
          </div>
          
          <el-menu
            :default-active="$route.path"
            class="menu-top"
            router
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
          >
            <el-menu-item index="/settings">
              <el-icon><Setting /></el-icon>
              <span>设置</span>
            </el-menu-item>
          </el-menu>
        </div>
      </el-aside>
      
      <el-main class="app-main">
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
}
#app {
  height: 100%;
}
</style>

<style scoped>
.app-container {
  height: 100vh;
  overflow: hidden;
}

.app-aside {
  background-color: var(--el-bg-color);
  border-right: 1px solid var(--el-border-color-light);
}

.aside-flex {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.app-logo {
  padding: 20px;
  text-align: center;
  border-bottom: 1px solid var(--el-border-color-light);
}
.app-logo h2 {
  margin: 0;
  font-size: 18px;
  color: var(--el-text-color-primary);
}

.menu-top {
  flex-grow: 1;
  border-right: none !important;
  overflow-y: auto;
}

.menu-bottom {
  flex-grow: 0;
  border-right: none !important;
  border-top: 1px solid var(--el-border-color-light);
}

.app-main {
  padding: 20px;
  background-color: var(--el-fill-color-light);
  overflow-y: auto;
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>