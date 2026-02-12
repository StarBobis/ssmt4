<script setup lang="ts">
import { appSettings } from '../store'
import { open as openDialog } from '@tauri-apps/plugin-dialog';
import { openUrl } from '@tauri-apps/plugin-opener';
import { useI18n } from 'vue-i18n';
import { getVersion } from '@tauri-apps/api/app';
import { ref, onMounted } from 'vue';

const { t } = useI18n()
const appVersion = ref('1.0.0');

onMounted(async () => {
  try {
    appVersion.value = await getVersion();
  } catch (e) {
    console.error('Failed to get version:', e);
  }
});

const selectCacheDir = async () => {
  const selected = await openDialog({
    directory: true,
    multiple: false,
    title: '选择 SSMT 缓存文件夹'
  });

  if (selected && typeof selected === 'string') {
    appSettings.cacheDir = selected;
  }
};
</script>

<template>
  <div class="settings-page">
    <div class="settings-layout">
      <!-- 左侧：主要设置区域 -->
      <div class="settings-main">
        <div class="settings-section">
          <div class="section-title">
            <span class="title-text">常规设置</span>
            <span class="title-desc">语言、存储与其他偏好</span>
          </div>
          <div class="glass-card">
             <el-form label-position="top" size="large">
              <el-row :gutter="24">
                <!-- 语言 -->
                <el-col :span="12">
                   <el-form-item :label="t('settings.language')">
                      <el-select v-model="appSettings.locale" placeholder="Select language" class="glass-input">
                        <el-option label="简体中文" value="zhs" />
                        <el-option label="繁體中文" value="zht" />
                        <el-option label="English" value="en" />
                      </el-select>
                   </el-form-item>
                </el-col>
                <!-- GitHub Token -->
                <el-col :span="12">
                   <el-form-item label="GitHub Token">
                      <el-input 
                        v-model="appSettings.githubToken" 
                        placeholder="可选: 此API Token用于提高请求限额" 
                        type="password"
                        show-password 
                        class="glass-input"
                      />
                   </el-form-item>
                </el-col>
              </el-row>

              <!-- 缓存文件夹 -->
              <el-form-item label="SSMT 缓存文件夹">
                 <div class="path-selector">
                    <el-input 
                      v-model="appSettings.cacheDir" 
                      placeholder="请选择缓存文件夹路径" 
                      class="glass-input" 
                      readonly
                    />
                    <el-button class="glass-button" @click="selectCacheDir">更改目录</el-button>
                 </div>
              </el-form-item>
             </el-form>
          </div>
        </div>

        <div class="settings-section">
          <div class="section-title">
             <span class="title-text">外观与视觉</span>
             <span class="title-desc">自定义界面的透明度与模糊效果</span>
          </div>
          <div class="glass-card">
              <el-form label-position="top">
                 <div class="slider-group">
                    <div class="slider-item">
                       <span class="slider-label">背景不透明度 (Opacity)</span>
                       <el-slider 
                          v-model="appSettings.contentOpacity" 
                          :min="0" :max="1" :step="0.01" 
                          show-input 
                          class="glass-slider"
                        />
                    </div>
                    <div class="slider-item">
                       <span class="slider-label">背景模糊度 (Blur)</span>
                       <el-slider 
                         v-model="appSettings.contentBlur" 
                         :min="0" :max="50" :step="1" 
                         show-input
                         class="glass-slider" 
                        />
                    </div>
                 </div>
              </el-form>
          </div>
        </div>

        <div class="settings-section">
           <div class="section-title">
             <span class="title-text">界面模块</span>
             <span class="title-desc">选择需要在左侧导航栏显示的功能</span>
           </div>
           
           <div class="glass-card switches-card">
              <div class="switch-row">
                 <div class="switch-info">
                    <span class="sw-label">Mod 管理页面</span>
                 </div>
                 <el-switch v-model="appSettings.showMods" style="--el-switch-on-color: #409eff;" />
              </div>
              
              <div class="divider"></div>

              <div class="switch-row">
                 <div class="switch-info">
                    <span class="sw-label">常用网址页面</span>
                 </div>
                 <el-switch v-model="appSettings.showWebsites" style="--el-switch-on-color: #409eff;" />
              </div>

               <div class="divider"></div>

              <div class="switch-row">
                 <div class="switch-info">
                     <span class="sw-label">使用文档页面</span>
                 </div>
                 <el-switch v-model="appSettings.showDocuments" style="--el-switch-on-color: #409eff;" />
              </div>
           </div>
        </div>
      </div>

      <!-- 右侧：关于信息区域 (精简版) -->
      <div class="settings-sidebar">
         <div class="about-container">
             <div class="app-logo-mini">
                <img src="/icon.png" class="logo-image-mini" alt="App Logo" />
             </div>
             <div class="app-info-mini">
               <h2 class="app-name-mini">SSMT 4</h2>
               <span class="app-version-mini">v{{ appVersion }}</span>
             </div>
             
             <div class="divider-mini"></div>
             
             <div class="link-group-mini">
                <a href="#" @click.prevent="openUrl('https://github.com/StarBobis/ssmt4')" class="link-item-mini">
                  GitHub地址
                </a>
                <a href="#" @click.prevent="openUrl('https://github.com/StarBobis/ssmt4/issues')" class="link-item-mini">
                  提交问题反馈
                </a>
                <a href="#" @click.prevent="openUrl('https://github.com/StarBobis/ssmt4/releases')" class="link-item-mini">
                  检查版本更新
                </a>
             </div>

             <div class="copyright-mini">
                GPL-3.0 License<br>&copy; 2026 StarBobis
             </div>
         </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 整体布局容器 */
.settings-page {
  height: 100%;
  width: 100%;
  padding: 40px 60px; /* 两侧增加留白，更具呼吸感 */
  box-sizing: border-box;
  overflow-y: auto;
}

.settings-layout {
  display: flex;
  gap: 60px; /* 增加栏间距，区分主次 */
  max-width: 1400px; /* 允许更宽的布局 */
  margin: 0 auto;
  align-items: flex-start;
}

/* 左侧主内容 - 绝对重心 */
.settings-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 32px;
  min-width: 0; /* 防止Flex子项溢出 */
}

.settings-section {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

/* 标题样式 */
.section-title {
  display: flex;
  flex-direction: column;
  margin-left: 8px;
  margin-bottom: 8px;
}

.title-text {
  font-size: 24px; /* 更大的标题 */
  font-weight: 300; /* 更细的字重，现代感 */
  color: #fff;
  letter-spacing: 1px;
}

.title-desc {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 6px;
}

/* 通用玻璃卡片 */
.glass-card {
  background-color: rgba(255, 255, 255, 0.03);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
  border: 1px solid rgba(255, 255, 255, 0.05); /* 更淡的边框 */
  border-radius: 20px; /* 更大的圆角 */
  padding: 32px; /* 更大的内衬 */
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.1);
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.glass-card:hover {
  background-color: rgba(255, 255, 255, 0.06);
  border-color: rgba(255, 255, 255, 0.1);
  transform: translateY(-2px);
  box-shadow: 0 12px 48px rgba(0, 0, 0, 0.15);
}

/* 右侧侧边栏 - 极简/静默模式 */
.settings-sidebar {
  width: 160px; /* 缩窄宽度 */
  flex-shrink: 0;
  /* position: sticky;  REMOVED: 让它随页面滚动 */
  /* top: 60px; */
  padding-top: 10px; /* 稍微与左侧标题对齐 */
}

.about-container {
  display: flex;
  flex-direction: column;
  align-items: flex-start; /* 左对齐 */
  opacity: 0.6; /* 默认低不透明度，不抢视觉 */
  transition: opacity 0.3s ease;
}

.about-container:hover {
  opacity: 1; /* hover时高亮 */
}

.app-logo-mini {
  margin-bottom: 12px;
}
.logo-image-mini {
  width: 56px; /* 稍微大一点点 */
  height: 56px;
  border-radius: 14px;
  box-shadow: 0 4px 12px rgba(0,0,0,0.2);
}

.app-info-mini {
  display: flex;
  flex-direction: column;
}

.app-name-mini {
  margin: 0;
  font-size: 20px;
  font-weight: 500;
  color: #fff;
  letter-spacing: 0.5px;
}

.app-version-mini {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  font-family: monospace;
}

.divider-mini {
  width: 24px;
  height: 2px;
  background: rgba(255, 255, 255, 0.2);
  margin: 24px 0;
}

.link-group-mini {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.link-item-mini {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.6);
  text-decoration: none;
  display: flex;
  align-items: center;
  gap: 10px;
  transition: color 0.2s;
}

.link-item-mini:hover {
  color: #409eff;
}

.link-icon {
  display: inline-flex;
  width: 20px;
  justify-content: center;
  font-size: 14px;
  font-weight: bold;
  opacity: 0.7;
}

.copyright-mini {
  margin-top: 40px;
  font-size: 12px;
  color: rgba(255, 255, 255, 0.3);
  line-height: 1.6;
}

/* Form Styles Override */
:deep(.el-form-item__label) {
  color: rgba(255, 255, 255, 0.8) !important;
  font-weight: 500;
}

/* Glass Inputs */
.glass-input :deep(.el-input__wrapper),
.glass-input :deep(.el-select__wrapper) {
  background-color: rgba(0, 0, 0, 0.2) !important;
  box-shadow: none !important;
  border: 1px solid rgba(255,255,255,0.1);
  border-radius: 8px;
  color: #fff;
}
.glass-input :deep(.el-input__inner) {
  color: #fff;
}
.glass-input :deep(.el-input__wrapper.is-focus) {
  border-color: rgba(255,255,255,0.3);
  background-color: rgba(0, 0, 0, 0.3) !important;
}

/* 路径选择器组合 */
.path-selector {
  display: flex;
  gap: 12px;
  width: 100%;
  align-items: center;
}

/* 让输入框占据剩余空间 */
.path-selector :deep(.el-input) {
  flex: 1;
}

.glass-button {
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: rgba(255,255,255,0.9);
  border-radius: 8px;
  padding: 8px 16px;
  height: auto; /* Allow height to adjust */
}
.glass-button:hover {
  background: rgba(255, 255, 255, 0.15);
  border-color: rgba(255, 255, 255, 0.2);
  color: #fff;
}

/* Switches Card */
.switches-card {
  padding: 8px 32px; /* 减少上下内边距 */
}
.switch-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 0; /* 调整行间距 */
}
.sw-label {
  font-size: 15px;
  color: rgba(255,255,255,0.9);
  font-weight: 500;
}
.sw-desc {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.4);
}
.divider {
  height: 1px; /* 修复无法显示的问题 */
  background: rgba(255, 255, 255, 0.05);
  width: 100%;
}

/* 移除内部冲突的动画，交由 App.vue 的 Router Transition 统一管理 */
/*.settings-page {
  animation: slideUpFade 0.5s cubic-bezier(0.16, 1, 0.3, 1) forwards;
}*/

/*@keyframes slideUpFade {
  from { opacity: 0; transform: translateY(20px); }
  to { opacity: 1; transform: translateY(0); }
}*/
</style>
