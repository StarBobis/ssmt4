<script setup lang="ts">
import { gamesList, switchToGame, appSettings } from '../store';
import { useRouter } from 'vue-router';

// We can choose to keep the button navigation separate, 
// or allow clicking a game to "launch" it (which usually implies going to the workspace or just running it).
// The user request didn't specify what happens on click, but "switching to game" usually implies context switch.
// I'll stick to just setting active game. If they want to nav, they can use the TitleBar button or I can add nav.
// Usually a launcher goes to "details" or "dashboard" after selection.
// For now: Select + Switch to Workbench seems logical for a successful "Pick".
const router = useRouter();

const handleGameSelect = (game: any) => {
    switchToGame(game);
    // Optional: Auto-navigate to workbench after selection? 
    // The previous logic was "keep drawer open for a moment".
    // I'll stay on page to allow browsing, unless user wants otherwise.
    // Actually, "Drawer" implies temporary. "Page" implies persistent.
    // I'll just set the cached game.
};
</script>

<template>
    <div class="game-library-container">
        <div class="games-grid">
            <div 
                v-for="game in gamesList" 
                :key="game.name"
                class="game-card"
                :class="{ active: appSettings.currentConfigName === game.name }"
                @click="handleGameSelect(game)"
            >
                <div class="game-icon-wrapper">
                    <img :src="game.iconPath" class="game-icon" alt="icon" />
                    <div class="game-label">{{ game.name }}</div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.game-library-container {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    min-height: 100%;
    box-sizing: border-box;
    padding-top: 60px; /* TitleBar Safe Area */
    padding-bottom: 40px;
    
    /* Background moved to App.vue for global coverage including TitleBar */
    background: transparent; 
    
    overflow-y: auto;
    overflow-x: hidden; /* Prevent horizontal scrollbar caused by scaled breathing effects */
}

.games-grid {
    display: flex;
    flex-wrap: wrap; 
    justify-content: center;
    gap: 30px;
    padding: 20px 60px;
}

/* --- Crystal Icon Styles (Reused & Adapted) --- */

.game-card {
    position: relative;
    flex: 0 0 auto;
    display: flex;
    flex-direction: column;
    align-items: center;
    cursor: pointer;
    transition: transform 0.4s cubic-bezier(0.25, 1, 0.5, 1), opacity 0.4s ease, filter 0.4s ease;
    width: 110px; /* Slightly larger for grid */
    opacity: 0.85; /* More visible on page than in drawer */
    filter: brightness(0.8) grayscale(0.2); 
    transform-origin: center center;
}

.game-card:hover {
    transition: transform 0.15s ease-out, opacity 0.2s ease, filter 0.2s ease;
    transform: scale(1.1);
    opacity: 1;
    filter: none;
    z-index: 100;
}

.game-card.active {
    opacity: 1;
    filter: none;
    transform: scale(1.05); /* Slight highlight for active */
    z-index: 10;
}

.game-icon-wrapper {
    position: relative;
    width: 90px;
    height: 90px;

    /* Crystal filling texture */
    background: radial-gradient(circle at 50% 0%,
            rgba(255, 255, 255, 0.15) 0%,
            rgba(255, 255, 255, 0.05) 40%,
            rgba(255, 255, 255, 0.02) 100%);
    backdrop-filter: blur(3px);
    border: 1px solid rgba(255, 255, 255, 0.25);
    border-radius: 14px;
    padding: 3px;

    display: flex;
    align-items: center;
    justify-content: center;

    /* Constant radiating light */
    box-shadow:
        0 0 12px rgba(130, 200, 255, 0.15),
        inset 0 0 15px rgba(255, 255, 255, 0.1);

    transition: all 0.2s;
    overflow: hidden;
    animation: crystalPulse 5s ease-in-out infinite;
}

@keyframes crystalPulse {
    0%, 100% {
        box-shadow: 0 0 12px rgba(130, 200, 255, 0.15), inset 0 0 15px rgba(255, 255, 255, 0.1);
        border-color: rgba(255, 255, 255, 0.25);
    }
    50% {
        box-shadow: 0 0 22px rgba(130, 210, 255, 0.35), inset 0 0 22px rgba(255, 255, 255, 0.25);
        border-color: rgba(255, 255, 255, 0.5);
    }
}

/* Magic fluid/particle flow */
.game-icon-wrapper::before {
    content: "";
    position: absolute;
    top: -50%;
    left: -50%;
    width: 200%;
    height: 200%;
    background:
        conic-gradient(from 0deg at 50% 50%,
            transparent 0deg,
            rgba(255, 255, 255, 0.05) 40deg,
            rgba(100, 200, 255, 0.1) 90deg,
            transparent 135deg,
            rgba(255, 255, 255, 0.05) 200deg,
            transparent 360deg);
    filter: blur(15px);
    animation: magicRotate 7s linear infinite;
    z-index: 2;
    pointer-events: none;
    mix-blend-mode: screen;
}

@keyframes magicRotate {
    0% { transform: rotate(0deg); }
    100% { transform: rotate(360deg); }
}

/* Crystal reflection sheen */
.game-icon-wrapper::after {
    content: "";
    position: absolute;
    top: 0;
    left: -150%;
    width: 200%;
    height: 100%;
    background: linear-gradient(115deg,
            transparent 40%,
            rgba(255, 255, 255, 0.05) 45%,
            rgba(255, 255, 255, 0.4) 50%,
            rgba(255, 255, 255, 0.05) 55%,
            transparent 60%);
    transform: skewX(-20deg);
    pointer-events: none;
    z-index: 5;
    animation: subtleSheen 6s ease-in-out infinite;
}

@keyframes subtleSheen {
    0% { left: -150%; opacity: 0.3; }
    40% { left: 150%; opacity: 0.3; }
    100% { left: 150%; opacity: 0.3; }
}

.game-card:hover .game-icon-wrapper::after {
    left: 150%;
    transition: none; /* Reset for hover effect if needed, but keeping animation is fine */
}

/* Stronger hover effects */
.game-card:hover .game-icon-wrapper {
    transform: translateY(-2px);
    border-color: rgba(255, 255, 255, 0.6);
    box-shadow: 0 0 25px rgba(150, 220, 255, 0.4), inset 0 0 25px rgba(255, 255, 255, 0.3);
}

.game-card.active {
    opacity: 1;
    filter: none;
    transform: scale(1.2); /* Moderately larger */
    z-index: 100;
}

/* Radiating Warm White Breathing Light */
.game-card.active::before {
    content: "";
    position: absolute;
    top: 50%; left: 50%;
    /* Significantly increased range */
    width: 250%; height: 250%;
    transform: translate(-50%, -50%);
    
    /* Warm white radial spectrum */
    background: radial-gradient(
        circle closest-side, 
        rgba(255, 250, 230, 0.4) 0%,
        rgba(255, 240, 200, 0.25) 30%,
        rgba(255, 230, 180, 0.1) 60%,
        transparent 80%
    );
    
    z-index: -1;
    border-radius: 50%; /* Soft circular glow */
    filter: blur(20px);
    animation: radiateBreath 4s ease-in-out infinite;
    pointer-events: none; /* Critical: Prevent blocking clicks on adjacent items */
}

/* Remove the sharp second border */
.game-card.active::after {
    display: none;
}

/* No rotation, just pulsing outwards from center */
@keyframes radiateBreath {
    0%, 100% {
        opacity: 0.5;
        transform: translate(-50%, -50%) scale(0.9);
    }
    50% {
        opacity: 0.9;
        transform: translate(-50%, -50%) scale(1.1);
    }
}

.game-card.active .game-icon-wrapper {
    /* Hide the default white border so the rainbow shows nicely around it */
    border-color: rgba(255, 255, 255, 0.2); 
    /* Add an inner glow to blend with the outer rainbow */
    box-shadow: inset 0 0 20px rgba(255, 255, 255, 0.5);
}

.game-icon {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 12px;
    display: block;
    z-index: 1;
    position: relative;
}

.game-label {
    position: absolute;
    left: 0;
    bottom: 0;
    width: 100%;
    text-align: center;
    font-size: 11px;
    font-weight: 600;
    color: #fff;
    text-shadow: 0 1px 3px rgba(0, 0, 0, 0.9);
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(2px);
    padding: 3px 0;
    line-height: 1.2;
    z-index: 10;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    border-bottom-left-radius: 14px;
    border-bottom-right-radius: 14px;
}
</style>