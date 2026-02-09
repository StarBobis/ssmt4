import { createRouter, createWebHistory } from 'vue-router'
import Home from '../views/Home.vue'
import Workbench from '../views/Workbench.vue'
import Stickers from '../views/Stickers.vue'
import Websites from '../views/Websites.vue'
import Settings from '../views/Settings.vue'

const routes = [
  { path: '/', name: 'Home', component: Home },
  { path: '/workbench', name: 'Workbench', component: Workbench },
  { path: '/stickers', name: 'Stickers', component: Stickers },
  { path: '/websites', name: 'Websites', component: Websites },
  { path: '/settings', name: 'Settings', component: Settings },
]

const router = createRouter({
  history: createWebHistory(),
  routes,
})

export default router
