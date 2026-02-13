<script setup lang="ts">
import { ref, markRaw } from 'vue'
import { openUrl } from '@tauri-apps/plugin-opener';
import { 
  Tools, 
  MagicStick, 
  Monitor, 
  Box, 
  Download, 
  Share, 
  Link as LinkIcon 
} from '@element-plus/icons-vue'

interface LinkItem {
  label: string
  url: string
  desc: string
}

interface Category {
  title: string
  icon: any
  color: string
  items: LinkItem[]
}

const categories = ref<Category[]>([
  {
    title: '3Dmigoto',
    icon: markRaw(Tools),
    color: '#00ffff',
    items: [
      { label: '3Dmigoto Github', url: 'https://github.com/bo3b/3Dmigoto', desc: '3Dmigoto 官方仓库' },
      { label: '3D Fixs', url: 'https://github.com/DarkStarSword/3d-fixes', desc: '远古时期3D Fixs概念验证脚本' },
    ]
  },
  {
    title: 'AGMG 常用Mod工具集',
    icon: markRaw(MagicStick),
    color: '#ff00ff',
    items: [
      { label: 'AGMG Discord', url: 'https://discord.gg/agmg', desc: '全球最大的3Dmigoto技术交流服务器' },
    ]
  },
  {
    title: 'SSMT 常用工具集',
    icon: markRaw(Monitor),
    color: '#00ffaa',
    items: [
      { label: 'SSMT3 Github', url: 'https://github.com/StarBobis/SSMT', desc: 'SSMT3 官方Github仓库' },
      { label: 'TheHerta3 Github', url: 'https://github.com/StarBobis/TheHerta3', desc: 'SSMT3 配套Blender插件' },
      { label: 'SSMT-Documentation', url: 'https://starbobis.github.io/SSMT-Documents/', desc: 'SSMT3 官方使用手册' },
      { label: 'SSMT4 Github', url: 'https://github.com/StarBobis/ssmt4', desc: 'SSMT4 官方Github仓库' },
      { label: 'SSMT4-Documentation', url: 'https://starbobis.github.io/SSMT4-Documents/', desc: 'SSMT4 官方使用手册' },
      { label: 'SSMT Discord', url: 'https://discord.gg/hWVfR9W2wn', desc: 'SSMT防失联技术交流社区' },

    ]
  },
  {
    title: 'Blender 插件',
    icon: markRaw(Box),
    color: '#ffaa00',
    items: [
      { label: 'Blender Official', url: 'https://www.blender.org/', desc: '开源 3D 创作套件' },
    ]
  },
  {
    title: '模型获取',
    icon: markRaw(Download),
    color: '#0088ff',
    items: [
      { label: '模之屋 (Aplaybox)', url: 'https://www.aplaybox.com/', desc: '高质量 MMD 模型与动作库' },
      { label: 'BowlRoll', url: 'https://bowlroll.net/', desc: '日本 MM 资源分享站' },
      { label: 'DeviantArt', url: 'https://www.deviantart.com/', desc: '全球艺术与模型社区' }
    ]
  },
  {
    title: 'Mod 获取',
    icon: markRaw(Share),
    color: '#aa00ff',
    items: [
      { label: 'GameBanana', url: 'https://gamebanana.com/', desc: '最大的游戏 Mod 社区' },
      { label: 'Nexus Mods', url: 'https://www.nexusmods.com/', desc: '综合 Mod 下载与管理站' },
      { label: 'Hui站', url: 'https://huihui168.org/', desc: '不限速的免费Mod资源下载站' },

    ]
  },
  {
    title: '其它网站',
    icon: markRaw(LinkIcon),
    color: '#ffff00',
    items: [
      { label: 'Vue.js', url: 'https://vuejs.org/', desc: '渐进式 JavaScript 框架' },
      { label: 'Vite', url: 'https://vitejs.dev/', desc: '下一代前端构建工具' },
      { label: 'Element Plus', url: 'https://element-plus.org/', desc: 'Vue 3 UI 组件库' }
    ]
  }
])

const handleOpen = async (url: string) => {
  try {
    // 优先尝试 Tauri opener
    await openUrl(url);
  } catch (e) {
    console.warn('Tauri openUrl failed, falling back to window.open:', e);
    // Fallback for web / dev mode without backend
    window.open(url, '_blank');
  }
}
</script>

<template>
  <div class="websites-page">
    <div class="header-section">
      <h1 class="main-title">常用网址导航</h1>
      <p class="sub-title">精选工具与资源链接 · Useful Links & Resources</p>
    </div>

    <div class="grid-container">
      <div 
        v-for="(cat, index) in categories" 
        :key="index" 
        class="category-card"
        :style="{ '--theme-color': cat.color }"
      >
        <div class="card-header">
          <el-icon class="cat-icon" :size="24" :color="cat.color">
            <component :is="cat.icon" />
          </el-icon>
          <span class="cat-title" :style="{ color: cat.color }">{{ cat.title }}</span>
        </div>
        
        <div class="links-list">
          <div 
            v-for="(link, lIndex) in cat.items" 
            :key="lIndex" 
            class="link-item"
            @click="handleOpen(link.url)"
          >
            <div class="link-label">{{ link.label }}</div>
            <div class="link-desc">{{ link.desc }}</div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.websites-page {
  padding: 40px;
  color: #fff;
  min-height: 100%;
}

.header-section {
  margin-bottom: 40px;
  text-align: center;
}

.main-title {
  font-size: 36px;
  font-weight: 300;
  margin: 0 0 10px 0;
  letter-spacing: 2px;
  text-shadow: 0 0 20px rgba(255, 255, 255, 0.2);
}

.sub-title {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.5);
  letter-spacing: 1px;
}

.grid-container {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 24px;
}

.category-card {
  background: rgba(20, 20, 25, 0.55);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 12px;
  padding: 20px;
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
  backdrop-filter: blur(12px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.category-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: 0;
  width: 4px;
  height: 100%;
  background: var(--theme-color);
  opacity: 0.7;
  box-shadow: 0 0 15px var(--theme-color);
}

.category-card:hover {
  transform: translateY(-5px);
  background: rgba(30, 30, 45, 0.75);
  border-color: rgba(255, 255, 255, 0.15);
  box-shadow: 0 12px 30px rgba(0, 0, 0, 0.5), 0 0 20px var(--theme-color); /* Deep shadow + glow */
}

/* Header with icon */
.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
}

.cat-title {
  font-size: 18px;
  font-weight: 600;
  text-shadow: 0 0 10px rgba(0, 0, 0, 0.5);
}

.links-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.link-item {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.03);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.25, 0.8, 0.25, 1);
  border-left: 2px solid transparent;
}

.link-item:hover {
  background: rgba(255, 255, 255, 0.08);
  border-left-color: var(--theme-color);
  padding-left: 20px; /* Slide effect */
  box-shadow: 0 2px 8px rgba(0,0,0,0.2);
}

.link-label {
  font-size: 15px;
  font-weight: 500;
  color: #e0e0e0;
  margin-bottom: 4px;
  transition: color 0.2s;
}

.link-desc {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.link-item:hover .link-label {
  color: #fff;
  text-shadow: 0 0 8px var(--theme-color);
}
</style>
