<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import { gamesList, switchToGame, appSettings } from '../store';

const scrollContainer = ref<HTMLElement | null>(null);

const onWheel = (e: WheelEvent) => {
  if (scrollContainer.value) {
    e.preventDefault();
    scrollContainer.value.scrollLeft += e.deltaY;
  }
};

const handleGameSelect = (game: any) => {
  switchToGame(game);
};

</script>

<template>
  <div class="page-container home-container">
    <div class="games-slider-wrapper">
        <div 
          ref="scrollContainer" 
          class="games-slider"
          @wheel="onWheel"
        >
          <div 
            v-for="game in gamesList" 
            :key="game.name"
            class="game-card"
            :class="{ active: appSettings.currentConfigName === game.name }"
            @click="handleGameSelect(game)"
          >
            <div class="game-icon-wrapper">
              <img :src="game.iconPath" class="game-icon" alt="icon" />
            </div>
            <div class="game-label">{{ game.name }}</div>
          </div>
        </div>
    </div>
    
    <div class="current-info">
      <h2>当前配置: {{ appSettings.currentConfigName }}</h2>
    </div>
  </div>
</template>

<style scoped>
.home-container {
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  height: 100%;
}

.games-slider-wrapper {
  width: 100%;
  max-width: 1200px;
  padding: 20px 0;
}

.games-slider {
  display: flex;
  gap: 30px;
  overflow-x: auto;
  padding: 20px 40px;
  scroll-behavior: smooth;
  /* Hide scrollbar for cleaner look */
  scrollbar-width: none; 
  -ms-overflow-style: none;
}

.games-slider::-webkit-scrollbar {
  display: none;
}

.game-card {
  flex: 0 0 auto;
  width: 120px;
  display: flex;
  flex-direction: column;
  align-items: center;
  cursor: pointer;
  transition: transform 0.2s, opacity 0.2s;
  opacity: 0.7;
}

.game-card:hover {
  transform: scale(1.05);
  opacity: 0.9;
}

.game-card.active {
  opacity: 1;
  transform: scale(1.1);
}

.game-icon-wrapper {
  width: 100px;
  height: 100px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 16px;
  padding: 10px;
  backdrop-filter: blur(5px);
  border: 2px solid transparent;
  transition: border-color 0.3s;
}

.game-card.active .game-icon-wrapper {
  border-color: var(--el-color-primary);
  box-shadow: 0 0 15px rgba(0,0,0,0.5);
}

.game-icon {
  width: 100%;
  height: 100%;
  object-fit: contain;
}

.game-label {
  margin-top: 10px;
  font-size: 14px;
  font-weight: 600;
  text-align: center;
  color: #fff;
  text-shadow: 0 2px 4px rgba(0,0,0,0.8);
}

.current-info {
  margin-top: 40px;
  color: white;
  text-shadow: 0 2px 4px rgba(0,0,0,0.8);
}
</style>
