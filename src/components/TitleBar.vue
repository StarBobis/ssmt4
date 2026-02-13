<script setup lang="ts">
import { ref, reactive, onMounted, onUnmounted, computed } from 'vue';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useRouter, useRoute } from 'vue-router';
import { appSettings } from '../store';

const appWindow = getCurrentWindow();
const router = useRouter();
const route = useRoute();
const isMaximized = ref(false);

const checkMaximized = async () => {
    isMaximized.value = await appWindow.isMaximized();
};

let unlistenResize: (() => void) | null = null;

// --- Nav Button Logic ---
const STORAGE_KEY_NAV_ORDER = 'ssmt4_nav_order';

// interface NavButton {
//     id: string;
//     path: string;
//     label: string;
//     // We will render icon using conditional template or dynamic component if we extracted them
//     // For now, simpler to just use ID to switch in template or store SVG path
//     iconPath?: string; 
//     iconPolys?: string; // For complex SVGs
//     // Or just simple raw logic
// }

// We rely on stable IDs
const allNavItems = ref<any[]>([
    { id: 'home', path: '/', label: '主页', iconType: 'home' },
    { id: 'mods', path: '/mods', label: 'Mod管理', iconType: 'mods' },
    { id: 'websites', path: '/websites', label: '常用网址', iconType: 'websites' },
    { id: 'documents', path: '/documents', label: '使用文档', iconType: 'documents' },
]);

// Current order of IDs
const navOrder = ref<string[]>([]);

// Initialize Order
const loadOrder = () => {
    try {
        const stored = localStorage.getItem(STORAGE_KEY_NAV_ORDER);
        if (stored) {
            const parsed = JSON.parse(stored);
            if (Array.isArray(parsed)) {
                navOrder.value = parsed;
                return;
            }
        }
    } catch (e) { console.warn('Failed to load nav order', e); }
    
    // Default order
    navOrder.value = allNavItems.value.map(i => i.id);
};

const saveOrder = () => {
    localStorage.setItem(STORAGE_KEY_NAV_ORDER, JSON.stringify(navOrder.value));
};

// Computed display list
const displayItems = computed(() => {
    // strict order based on navOrder
    const map = new Map(allNavItems.value.map(i => [i.id, i]));
    const result: any[] = [];
    
    // Add items in order
    navOrder.value.forEach(id => {
        const item = map.get(id);
        if (item) {
             // Check visibility
             if (id === 'mods' && !appSettings.showMods) return;
             if (id === 'websites' && !appSettings.showWebsites) return;
             if (id === 'documents' && !appSettings.showDocuments) return;
             result.push(item);
        }
    });

    // Append any new items not in order (robustness)
    allNavItems.value.forEach(i => {
        if (!navOrder.value.includes(i.id)) {
             // Check visibility
             if (i.id === 'mods' && !appSettings.showMods) return;
             if (i.id === 'websites' && !appSettings.showWebsites) return;
             if (i.id === 'documents' && !appSettings.showDocuments) return;
             result.push(i);
        }
    });
    
    return result;
});

// Manual drag state to bypass Tauri drag restrictions
const navHoverId = ref<string | null>(null);
const navDraggingId = ref<string | null>(null);
const navManualState = reactive({
    active: false,
    startX: 0,
    startY: 0,
    hasMoved: false,
    itemId: null as string | null,
});

const resetNavDrag = () => {
    navManualState.active = false;
    navManualState.hasMoved = false;
    navManualState.itemId = null;
    navHoverId.value = null;
    navDraggingId.value = null;
    document.body.style.userSelect = '';
};

const applyNavReorder = (sourceId: string, targetId: string) => {
    const oldIndex = navOrder.value.indexOf(sourceId);
    const newIndex = navOrder.value.indexOf(targetId);
    if (oldIndex === -1 || newIndex === -1) return;
    const newOrder = [...navOrder.value];
    newOrder.splice(oldIndex, 1);
    newOrder.splice(newIndex, 0, sourceId);
    navOrder.value = newOrder;
    saveOrder();
};

const onNavMouseDown = (e: MouseEvent, item: any) => {
    if (e.button !== 0) return;
    navManualState.active = true;
    navManualState.startX = e.clientX;
    navManualState.startY = e.clientY;
    navManualState.hasMoved = false;
    navManualState.itemId = item.id;
    navDraggingId.value = item.id;
    document.addEventListener('mousemove', onNavMouseMove);
    document.addEventListener('mouseup', onNavMouseUp);
};

const onNavMouseMove = (e: MouseEvent) => {
    if (!navManualState.active || !navManualState.itemId) return;
    const dx = e.clientX - navManualState.startX;
    const dy = e.clientY - navManualState.startY;
    if (!navManualState.hasMoved && Math.hypot(dx, dy) > 3) {
        navManualState.hasMoved = true;
        document.body.style.userSelect = 'none';
    }

    if (navManualState.hasMoved) {
        const el = document.elementFromPoint(e.clientX, e.clientY) as HTMLElement | null;
        const btn = el?.closest?.('.nav-button') as HTMLElement | null;
        const targetId = btn?.dataset.navId || null;
        if (targetId && targetId !== navManualState.itemId) {
            navHoverId.value = targetId;
        } else {
            navHoverId.value = null;
        }
    }
};

const onNavMouseUp = (e: MouseEvent) => {
    document.removeEventListener('mousemove', onNavMouseMove);
    document.removeEventListener('mouseup', onNavMouseUp);

    if (navManualState.active && navManualState.hasMoved && navManualState.itemId && navHoverId.value && navHoverId.value !== navManualState.itemId) {
        applyNavReorder(navManualState.itemId, navHoverId.value);
    }

    resetNavDrag();
};

onMounted(async () => {
    checkMaximized();
    loadOrder();
    // Listen to resize event to update maximized state icon
    unlistenResize = await appWindow.onResized(() => {
        checkMaximized();
    });
});

onUnmounted(() => {
    if (unlistenResize) {
        unlistenResize();
    }
    document.removeEventListener('mousemove', onNavMouseMove);
    document.removeEventListener('mouseup', onNavMouseUp);
    resetNavDrag();
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
        if (window.history.length > 1) {
            router.back();
        } else {
            router.push('/');
        }
    } else {
        if (route.path === '/settings') {
            router.replace('/games');
        } else {
            router.push('/games');
        }
    }
};

const toggleSettingsPage = (e: MouseEvent) => {
    e.stopPropagation();
    if (route.path === '/settings') {
        if (window.history.length > 1) {
            router.back();
        } else {
            router.push('/');
        }
    } else {
        if (route.path === '/games') {
             router.replace('/settings');
        } else {
             router.push('/settings');
        }
    }
};

const navTo = (path: string) => {
    router.push(path);
};
</script>


<template>
  <div class="titlebar">
    <div class="nav-controls">
        <transition-group name="nav-list">
            <div 
                v-for="item in displayItems" 
                :key="item.id"
                class="nav-button" 
                :class="{ active: route.path === item.path, 'drag-hover': navHoverId === item.id, dragging: navDraggingId === item.id }" 
                :data-nav-id="item.id"
                @click="navTo(item.path)" 
                :title="item.label"
                @mousedown.prevent="onNavMouseDown($event, item)"
            >
                <!-- Icons -->
                <svg v-if="item.id === 'home'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"></path><polyline points="9 22 9 12 15 12 15 22"></polyline></svg>
                
                <svg v-if="item.id === 'mods'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"></path><polyline points="3.27 6.96 12 12.01 20.73 6.96"></polyline><line x1="12" y1="22.08" x2="12" y2="12"></line></svg>
                
                <svg v-if="item.id === 'websites'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path></svg>
                
                <svg v-if="item.id === 'documents'" xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"></path><polyline points="14 2 14 8 20 8"></polyline><line x1="16" y1="13" x2="8" y2="13"></line><line x1="16" y1="17" x2="8" y2="17"></line><polyline points="10 9 9 9 8 9"></polyline></svg>

                <span class="nav-text">{{ item.label }}</span>
            </div>
        </transition-group>
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
      <div class="control-button settings-btn" :class="{ active: route.path === '/settings' }" @click="toggleSettingsPage" title="Settings">
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
  background: rgba(0, 0, 0, 0.5); /* Darker background */
  backdrop-filter: blur(12px); /* stronger blur */
  transition: background 0.3s ease, backdrop-filter 0.3s ease;
}

.nav-controls {
    display: flex;
    align-items: center;
    height: 100%;
    padding-left: 0; 
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

/* Nav List Animation */
.nav-list-move, 
.nav-list-enter-active,
.nav-list-leave-active {
    transition: all 0.3s ease;
}
.nav-list-enter-from,
.nav-list-leave-to {
    opacity: 0;
    transform: translateX(-10px);
}
/* ensure leaving items are taken out of flow so others can move smoothly */
.nav-list-leave-active {
    position: absolute; 
}

.nav-button.drag-hover {
    background: rgba(64, 158, 255, 0.15);
    outline: 1px dashed #409eff;
}

.nav-button.dragging {
    opacity: 0.65;
}
</style>
