import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import GameLibrary from '../views/GameLibrary.vue'
import Workbench from '../views/Workbench.vue'
import Stickers from '../views/Stickers.vue'
import Websites from '../views/Websites.vue'
import Settings from '../views/Settings.vue'
import Documents from '../views/Documents.vue'

const routes = [
  { path: '/', name: 'Home', component: Home },
  { path: '/games', name: 'GameLibrary', component: GameLibrary },
  { path: '/workbench', name: 'Workbench', component: Workbench },
  { path: '/stickers', name: 'Stickers', component: Stickers },
  { path: '/websites', name: 'Websites', component: Websites },
  { path: '/settings', name: 'Settings', component: Settings },
  { path: '/documents', name: 'Documents', component: Documents },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
