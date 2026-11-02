<template>
  <div class="h-full flex flex-col px-6 pt-3 pb-6 overflow-y-auto">
    <!-- Top Header -->
    <div class="flex items-start justify-between mb-4 flex-shrink-0">
      <div>
        <div class="flex items-center gap-2.5">
          <h1 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
            <span>{{ t('stutterDetector.title') }}</span>
          </h1>
          <!-- Live connection badge -->
          <span
            v-if="status?.isConnected"
            class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-sky-400 border border-emerald-300 dark:border-emerald-500/30 shadow-xs"
          >
            <span class="w-2 h-2 rounded-full bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]"></span>
            {{ t('stutterDetector.connected') }} (UDP :{{ status.port }})
          </span>
          <span
            v-else-if="status?.isListening"
            class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-amber-50 text-amber-700 dark:bg-amber-950/40 dark:text-amber-300 border border-amber-300 dark:border-amber-800/60 shadow-xs"
          >
            <span class="w-2 h-2 rounded-full bg-amber-400 shadow-[0_0_8px_rgba(251,191,36,0.8)]"></span>
            {{ t('stutterDetector.listening') }} (:{{ status?.port || 49005 }})
          </span>
          <span
            v-else
            class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full text-xs font-semibold bg-gray-100 text-gray-600 dark:bg-slate-800/60 dark:text-slate-400 border border-gray-200 dark:border-slate-800"
          >
            <span class="w-2 h-2 rounded-full bg-gray-400"></span>
            {{ t('stutterDetector.idle') }}
          </span>
        </div>
        <p class="text-xs text-gray-500 dark:text-slate-400 mt-1 max-w-2xl">
          {{ t('stutterDetector.subtitle') }}
        </p>

        <!-- Telemetry Connection Hint (explains when UDP begins streaming) -->
        <div
          v-if="status?.isListening && !status?.isConnected"
          class="mt-2 inline-flex items-center gap-1.5 text-xs text-sky-600 dark:text-sky-400 font-medium"
        >
          <svg class="w-3.5 h-3.5 flex-shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{{ t('stutterDetector.telemetryHint') }}</span>
        </div>
      </div>

      <!-- Header actions -->
      <div class="flex items-center gap-2">
        <button
          type="button"
          @click="showSetupGuide = !showSetupGuide"
          class="px-3 py-1.5 text-xs font-medium rounded-xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-800/60 hover:bg-gray-50 dark:hover:bg-slate-800 hover:dark:border-sky-500/30 text-gray-700 dark:text-slate-300 transition-colors flex items-center gap-1.5"
        >
          <svg class="w-4 h-4 text-gray-500 dark:text-slate-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          <span>{{ showSetupGuide ? t('common.close') : t('stutterDetector.howToConnect') }}</span>
        </button>

        <button
          type="button"
          @click="handleToggleMonitor"
          :class="[
            'px-3.5 py-1.5 text-xs font-semibold rounded-xl transition-all flex items-center gap-1.5 shadow-sm',
            status?.isListening
              ? 'bg-red-50 text-red-600 hover:bg-red-100 dark:bg-red-950/40 dark:text-red-300 border border-red-200 dark:border-red-900/60'
              : 'bg-sky-500 hover:bg-sky-400 text-white shadow-sky-500/20 active:scale-95'
          ]"
        >
          <span>{{ status?.isListening ? t('stutterDetector.stopMonitor') : t('stutterDetector.startMonitor') }}</span>
        </button>
      </div>
    </div>

    <!-- Collapsible Setup Guide -->
    <div
      v-if="showSetupGuide || (!status?.isConnected && events.length === 0)"
      class="mb-4 p-4 rounded-2xl border border-sky-200 dark:border-slate-800 bg-sky-50/70 dark:bg-slate-900/70 text-xs text-sky-900 dark:text-slate-200 flex-shrink-0 shadow-sm"
    >
      <div class="flex items-start justify-between gap-3 mb-2">
        <div class="font-bold text-sm text-sky-950 dark:text-white flex items-center gap-2">
          <svg class="w-4 h-4 text-emerald-500 dark:text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
          </svg>
          {{ t('stutterDetector.setupTitle') }}
        </div>
        <button
          type="button"
          @click="showSetupGuide = false"
          class="text-gray-400 hover:text-white text-sm"
        >
          ✕
        </button>
      </div>
      <ol class="list-decimal list-inside space-y-1 text-slate-700 dark:text-slate-300 leading-relaxed">
        <li>{{ t('stutterDetector.setupStep1') }}</li>
        <li>{{ t('stutterDetector.setupStep2') }}</li>
        <li>{{ t('stutterDetector.setupStep3', { port: status?.port || 49005 }) }}</li>
      </ol>
      <p class="mt-2 text-[11px] text-sky-500 dark:text-sky-400/90">
        {{ t('stutterDetector.setupNote') }}
      </p>
    </div>

    <!-- Live Performance Cards (Executive HUD: 5 Benchmark Telemetry Cards) -->
    <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-5 gap-3 mb-4 flex-shrink-0">
      <!-- 1. Real-Time Live FPS -->
      <div class="p-3.5 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm hover:dark:border-sky-500/30 transition-all flex flex-col justify-between">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-bold text-gray-500 dark:text-slate-400 uppercase tracking-wider">
            {{ t('stutterDetector.liveFps') }}
          </div>
          <span
            v-if="status?.isConnected"
            class="px-2 py-0.5 rounded-full text-[10px] font-semibold"
            :class="status.currentFps >= 30 ? 'bg-emerald-500/15 text-emerald-400 border border-emerald-500/30' : 'bg-amber-500/15 text-amber-400 border border-amber-500/30'"
          >
            {{ status.currentFps >= 30 ? 'Smooth' : 'Lagging' }}
          </span>
          <span v-else class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-gray-500/10 text-gray-400 border border-gray-500/20">
            Idle
          </span>
        </div>
        <div class="mt-2 flex items-baseline gap-1.5">
          <span
            :class="[
              'text-2xl font-extrabold font-mono tracking-tight',
              !status?.isConnected ? 'text-gray-400 dark:text-slate-500' :
              status.currentFps >= 30 ? 'text-sky-500 dark:text-sky-400' :
              status.currentFps >= 15 ? 'text-amber-600 dark:text-amber-400' :
              'text-red-600 dark:text-red-400'
            ]"
          >
            {{ status?.isConnected ? status.currentFps.toFixed(1) : '--' }}
          </span>
          <span class="text-xs font-semibold text-gray-500 dark:text-slate-400">FPS</span>
        </div>
        <div class="mt-1 text-[10px] text-gray-500 dark:text-slate-400 truncate">
          <span v-if="status?.isConnected">
            {{ status.currentFps >= (status.thresholdFps || 15) ? t('stutterDetector.statusSmooth') : t('stutterDetector.statusLagging') }}
          </span>
          <span v-else>{{ t('stutterDetector.waitingData') }}</span>
        </div>
      </div>

      <!-- 2. Benchmark Average & 1% Low FPS -->
      <div class="p-3.5 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm hover:dark:border-sky-500/30 transition-all flex flex-col justify-between">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-bold text-gray-500 dark:text-slate-400 uppercase tracking-wider">
            {{ t('stutterDetector.avgFps') }}
          </div>
          <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-emerald-500/15 text-emerald-400 border border-emerald-500/30">
            {{ t('stutterDetector.onePercentLow') }}
          </span>
        </div>
        <div class="mt-2 flex items-baseline gap-1.5">
          <span
            :class="[
              'text-2xl font-extrabold font-mono tracking-tight',
              status?.isConnected && (status.avgFps || 0) > 0 ? 'text-emerald-500 dark:text-emerald-400' : 'text-gray-400 dark:text-slate-500'
            ]"
          >
            {{ status?.isConnected && (status.avgFps || 0) > 0 ? status.avgFps.toFixed(1) : '--' }}
          </span>
          <span class="text-xs font-semibold text-gray-500 dark:text-slate-400">FPS</span>
        </div>
        <div class="mt-1 text-[10px] text-gray-500 dark:text-slate-400 flex items-center justify-between font-mono">
          <span>1% Low:</span>
          <strong class="text-emerald-600 dark:text-emerald-400">
            {{ status?.isConnected && (status.onePercentLowFps || 0) > 0 ? status.onePercentLowFps.toFixed(1) : '--' }} FPS
          </strong>
        </div>
      </div>

      <!-- 3. Range (Min & Max FPS) -->
      <div class="p-3.5 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm hover:dark:border-sky-500/30 transition-all flex flex-col justify-between">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-bold text-gray-500 dark:text-slate-400 uppercase tracking-wider">
            {{ t('stutterDetector.minMaxRange') }}
          </div>
          <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-sky-500/15 text-sky-400 border border-sky-500/30">
            Range
          </span>
        </div>
        <div class="mt-2 flex items-center justify-between font-mono">
          <div>
            <div class="text-[9px] uppercase text-slate-400 font-sans">{{ t('stutterDetector.minFps') }}</div>
            <div class="text-lg font-extrabold text-red-500 dark:text-red-400">
              {{ status?.isConnected && (status.minFps || 0) > 0 ? status.minFps.toFixed(1) : '--' }}
            </div>
          </div>
          <div class="text-right">
            <div class="text-[9px] uppercase text-slate-400 font-sans">{{ t('stutterDetector.maxFps') }}</div>
            <div class="text-lg font-extrabold text-sky-500 dark:text-sky-400">
              {{ status?.isConnected && (status.maxFps || 0) > 0 ? status.maxFps.toFixed(1) : '--' }}
            </div>
          </div>
        </div>
        <div class="mt-1 text-[10px] text-gray-500 dark:text-slate-400 truncate">
          {{ t('stutterDetector.avgFpsNotice') }}
        </div>
      </div>

      <!-- 4. CPU vs GPU Frame Time -->
      <div class="p-3.5 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm hover:dark:border-sky-500/30 transition-all flex flex-col justify-between">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-bold text-gray-500 dark:text-slate-400 uppercase tracking-wider">
            {{ t('stutterDetector.renderTiming') }}
          </div>
          <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-blue-500/15 text-blue-400 border border-blue-500/30">
            ms
          </span>
        </div>
        <div class="mt-1 flex items-center justify-between text-xs font-mono font-bold">
          <span class="text-blue-600 dark:text-blue-400">CPU: {{ status?.isConnected ? status.cpuTimeMs.toFixed(1) : '0' }} ms</span>
          <span class="text-purple-600 dark:text-purple-400">GPU: {{ status?.isConnected ? status.gpuTimeMs.toFixed(1) : '0' }} ms</span>
        </div>
        <!-- Balance Bar -->
        <div class="mt-1.5 w-full bg-gray-100 dark:bg-slate-800 h-1.5 rounded-full overflow-hidden flex">
          <div
            class="bg-blue-500 h-full transition-all duration-300"
            :style="{ width: `${cpuPercent}%` }"
          ></div>
          <div
            class="bg-purple-500 h-full transition-all duration-300"
            :style="{ width: `${100 - cpuPercent}%` }"
          ></div>
        </div>
        <div class="mt-1 text-[10px] text-gray-500 dark:text-slate-400 truncate">
          {{ bottleneckHint }}
        </div>
      </div>

      <!-- 5. Total Drops Recorded -->
      <div class="p-3.5 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm hover:dark:border-sky-500/30 transition-all flex flex-col justify-between">
        <div class="flex items-center justify-between">
          <div class="text-[10px] font-bold text-gray-500 dark:text-slate-400 uppercase tracking-wider">
            {{ t('stutterDetector.totalDrops') }}
          </div>
          <span class="px-2 py-0.5 rounded-full text-[10px] font-semibold bg-amber-500/15 text-amber-400 border border-amber-500/30">
            &lt; {{ selectedThreshold }} FPS
          </span>
        </div>
        <div class="mt-2 flex items-baseline gap-1.5">
          <span class="text-2xl font-extrabold font-mono text-gray-900 dark:text-white">
            {{ events.length }}
          </span>
          <span class="text-xs font-semibold text-gray-500 dark:text-slate-400">{{ t('stutterDetector.eventsCount') }}</span>
        </div>
        <div class="mt-1 text-[10px] text-gray-500 dark:text-slate-400 truncate">
          {{ lowestFpsRecorded ? `${t('stutterDetector.lowestDrop')}: ${lowestFpsRecorded.toFixed(1)} FPS` : t('stutterDetector.noDropsYet') }}
        </div>
      </div>
    </div>

    <!-- Telemetry Waveform Bezier Chart -->
    <div class="mb-4 p-4 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm flex-shrink-0">
      <div class="flex items-center justify-between mb-3">
        <div>
          <h3 class="text-xs font-bold text-gray-900 dark:text-white uppercase tracking-wider flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-sky-400 shadow-[0_0_6px_rgba(56,189,248,0.8)]"></span>
            <span>Real-Time Framerate Waveform</span>
          </h3>
          <p class="text-[11px] text-gray-500 dark:text-slate-400">
            Continuous real-time performance telemetry & drop threshold curve
          </p>
        </div>
        <div class="flex items-center gap-3 text-[11px] font-mono">
          <span class="flex items-center gap-1.5 text-sky-500 dark:text-sky-400">
            <span class="w-2.5 h-1 bg-sky-400 rounded-full inline-block"></span>
            Live ({{ (status?.currentFps || 0).toFixed(0) }} FPS)
          </span>
          <span class="flex items-center gap-1.5 text-red-500 dark:text-red-400">
            <span class="w-2.5 h-0.5 border-b-2 border-dashed border-red-400 inline-block"></span>
            &lt; {{ selectedThreshold }} FPS
          </span>
        </div>
      </div>

      <!-- SVG Chart Canvas -->
      <div class="w-full h-28 relative overflow-hidden rounded-xl bg-gray-50 dark:bg-slate-950/80">
        <svg class="w-full h-full" viewBox="0 0 500 100" preserveAspectRatio="none">
          <defs>
            <linearGradient id="telemetryWaveformGlow" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#38BDF8" stop-opacity="0.35" />
              <stop offset="50%" stop-color="#0284C7" stop-opacity="0.12" />
              <stop offset="100%" stop-color="#0369A1" stop-opacity="0.0" />
            </linearGradient>
            <filter id="telemetryGlowFilter" x="-20%" y="-20%" width="140%" height="140%">
              <feDropShadow dx="0" dy="0" stdDeviation="3" flood-color="#38BDF8" flood-opacity="0.6"/>
            </filter>
          </defs>

          <!-- Grid Lines -->
          <line x1="0" y1="20" x2="500" y2="20" stroke="rgba(255,255,255,0.04)" stroke-dasharray="4,4" />
          <line x1="0" y1="50" x2="500" y2="50" stroke="rgba(255,255,255,0.04)" stroke-dasharray="4,4" />
          <line x1="0" y1="80" x2="500" y2="80" stroke="rgba(255,255,255,0.04)" stroke-dasharray="4,4" />

          <!-- Threshold Red Line -->
          <line
            x1="0"
            :y1="thresholdY"
            x2="500"
            :y2="thresholdY"
            stroke="#EF4444"
            stroke-width="1.2"
            stroke-dasharray="4,3"
            opacity="0.75"
          />

          <!-- Waveform Area Gradient Fill -->
          <path :d="chartAreaPath" fill="url(#telemetryWaveformGlow)" />

          <!-- Waveform Bezier Line -->
          <path
            :d="chartLinePath"
            fill="none"
            stroke="#38BDF8"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            filter="url(#telemetryGlowFilter)"
          />

          <!-- Latest Value Vertex Dot -->
          <circle
            :cx="latestPointX"
            :cy="latestPointY"
            r="4"
            fill="#38BDF8"
            stroke="#0f172a"
            stroke-width="2"
          />
          <circle
            :cx="latestPointX"
            :cy="latestPointY"
            r="8"
            fill="none"
            stroke="#38BDF8"
            stroke-width="1"
            opacity="0.5"
            
          />
        </svg>

        <!-- Right axis labels -->
        <div class="absolute right-2 top-1 text-[9px] font-mono text-slate-500">80 FPS</div>
        <div class="absolute right-2 top-1/2 -translate-y-1/2 text-[9px] font-mono text-slate-500">40 FPS</div>
        <div class="absolute right-2 bottom-1 text-[9px] font-mono text-slate-500">0 FPS</div>
      </div>
    </div>

    <!-- Filter & Simulation Controls -->
    <div class="flex flex-wrap items-center justify-between gap-3 mb-4 p-3 rounded-2xl border border-gray-200 dark:border-slate-800 bg-gray-50 dark:bg-slate-900/70 flex-shrink-0">
      <div class="flex items-center gap-3">
        <!-- Threshold Selector -->
        <div class="flex items-center gap-1.5 text-xs text-gray-700 dark:text-slate-300">
          <span class="font-medium">{{ t('stutterDetector.triggerThreshold') }}:</span>
          <select
            v-model.number="selectedThreshold"
            @change="handleThresholdChange"
            class="px-2.5 py-1 text-xs rounded-xl border border-gray-300 dark:border-slate-800 bg-white dark:bg-slate-800/60 text-gray-900 dark:text-white font-mono outline-none focus:border-sky-500/50"
          >
            <option :value="10">&lt; 10 FPS (Extreme)</option>
            <option :value="15">&lt; 15 FPS (Default)</option>
            <option :value="20">&lt; 20 FPS (Sensitive)</option>
            <option :value="25">&lt; 25 FPS (Very Sensitive)</option>
          </select>
        </div>
      </div>

      <!-- Test simulation & clear buttons -->
      <div class="flex items-center gap-2">
        <div class="relative inline-block text-left">
          <button
            type="button"
            @click="showSimMenu = !showSimMenu"
            class="px-3 py-1.5 text-xs font-semibold rounded-xl border border-gray-300 dark:border-slate-800 bg-white dark:bg-slate-800/60 hover:bg-gray-50 dark:hover:bg-slate-800 hover:dark:border-sky-500/30 text-gray-700 dark:text-slate-200 transition-colors flex items-center gap-1.5"
          >
            <svg class="w-3.5 h-3.5 text-sky-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
            </svg>
            <span>{{ t('stutterDetector.simulateTestDrop') }}</span>
            <svg class="w-3.5 h-3.5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7" />
            </svg>
          </button>
          <!-- Dropdown -->
          <div
            v-if="showSimMenu"
            class="absolute right-0 mt-1 w-64 rounded-xl shadow-xl bg-white dark:bg-slate-800/95 border border-gray-200 dark:border-slate-700 py-1.5 z-50 text-xs overflow-hidden backdrop-blur-md"
          >
            <button
              @click="triggerSimulate('vram')"
              class="w-full text-left px-3.5 py-2 hover:bg-gray-100 dark:hover:bg-slate-700/60 text-gray-800 dark:text-slate-200 flex items-center gap-2"
            >
              <span class="w-2.5 h-2.5 rounded-sm bg-purple-500 flex-shrink-0"></span>
              <span>VRAM Overload (8.4 FPS)</span>
            </button>
            <button
              @click="triggerSimulate('cpu')"
              class="w-full text-left px-3.5 py-2 hover:bg-gray-100 dark:hover:bg-slate-700/60 text-gray-800 dark:text-slate-200 flex items-center gap-2"
            >
              <span class="w-2.5 h-2.5 rounded-sm bg-blue-500 flex-shrink-0"></span>
              <span>CPU Bottleneck (10.8 FPS)</span>
            </button>
            <button
              @click="triggerSimulate('gpu')"
              class="w-full text-left px-3.5 py-2 hover:bg-gray-100 dark:hover:bg-slate-700/60 text-gray-800 dark:text-slate-200 flex items-center gap-2"
            >
              <span class="w-2.5 h-2.5 rounded-sm bg-red-500 flex-shrink-0"></span>
              <span>GPU Overload (11.2 FPS)</span>
            </button>
            <button
              @click="triggerSimulate('scenery')"
              class="w-full text-left px-3.5 py-2 hover:bg-gray-100 dark:hover:bg-slate-700/60 text-gray-800 dark:text-slate-200 flex items-center gap-2"
            >
              <span class="w-2.5 h-2.5 rounded-sm bg-amber-500 flex-shrink-0"></span>
              <span>Scenery Tile Loading (9.5 FPS)</span>
            </button>
          </div>
        </div>

        <button
          type="button"
          :disabled="events.length === 0 && (!status || (status.totalDropsRecorded === 0 && (status.avgFps || 0) === 0))"
          @click="handleClearEvents"
          class="px-3 py-1.5 text-xs font-medium rounded-xl border border-red-200 dark:border-red-900/60 bg-red-50/50 dark:bg-red-950/20 text-red-600 dark:text-red-400 hover:bg-red-100 dark:hover:bg-red-950/40 transition-colors disabled:opacity-40 flex items-center gap-1.5"
        >
          <svg class="w-3.5 h-3.5 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
          <span>{{ t('stutterDetector.clear') }}</span>
        </button>
      </div>
    </div>

    <!-- Feed / Timeline List -->
    <div class="flex-1 space-y-3">
      <div v-if="events.length === 0" class="py-16 flex flex-col items-center justify-center text-center">
        <div class="w-12 h-12 rounded-full bg-emerald-50 dark:bg-emerald-950/40 text-emerald-500 dark:text-emerald-400 border border-emerald-300 dark:border-emerald-500/30 flex items-center justify-center mb-3">
          <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
          </svg>
        </div>
        <h3 class="text-sm font-bold text-gray-900 dark:text-white">
          {{ t('stutterDetector.noDropsTitle') }}
        </h3>
        <p class="text-xs text-gray-500 dark:text-slate-400 mt-1 max-w-sm">
          {{ t('stutterDetector.noDropsDesc') }}
        </p>
      </div>

      <!-- Event Cards -->
      <div
        v-for="ev in events"
        :key="ev.id"
        class="p-4 rounded-2xl border border-gray-200 dark:border-slate-800 bg-white dark:bg-slate-900/70 shadow-sm hover:dark:border-[#2A3744] transition-all"
      >
        <div class="flex items-start justify-between gap-3">
          <!-- Main Left -->
          <div class="space-y-1.5 flex-1 min-w-0">
            <!-- Badges line -->
            <div class="flex flex-wrap items-center gap-2 text-xs">
              <!-- Timestamp -->
              <span class="font-mono text-gray-500 dark:text-slate-400 font-semibold text-[11px]">
                {{ ev.timeFormatted }}
              </span>

              <!-- FPS Drop badge -->
              <span class="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-full font-bold font-mono text-red-700 dark:text-red-300 bg-red-100 dark:bg-red-950/60 border border-red-200 dark:border-red-900/60 text-[11px]">
                <span class="w-1.5 h-1.5 rounded-full bg-red-500"></span>
                <span>{{ ev.lowestFps }} FPS</span>
                <span class="text-[10px] opacity-75 font-normal">({{ t('stutterDetector.droppedFrom', { base: ev.baselineFps }) }})</span>
              </span>

              <!-- Duration badge -->
              <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[11px] font-mono text-gray-700 dark:text-slate-300 bg-gray-100 dark:bg-slate-800/60 border border-gray-200 dark:border-slate-800">
                <svg class="w-3 h-3 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span>{{ ev.durationSeconds }}s</span>
              </span>

              <!-- Cause Category Badge -->
              <span :class="['inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[11px] font-semibold', causeBadgeClass(ev.primaryCause)]">
                {{ getEventTitle(ev) }}
              </span>
            </div>

            <!-- Description -->
            <p class="text-xs text-gray-700 dark:text-slate-300 leading-relaxed pt-0.5">
              {{ getEventDesc(ev) }}
            </p>

            <!-- Suggestion / Solution Box -->
            <div class="mt-2 p-2.5 rounded-xl bg-amber-50/70 dark:bg-[#1B221B] border border-amber-200/80 dark:border-emerald-900/40 text-xs text-amber-900 dark:text-emerald-300/90 flex items-start gap-2">
              <svg class="w-4 h-4 text-amber-600 dark:text-emerald-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
              </svg>
              <div class="leading-relaxed">
                <strong class="font-bold text-amber-950 dark:text-emerald-300">{{ t('stutterDetector.solutionHeader') }}:</strong>
                {{ getEventSugg(ev) }}
              </div>
            </div>

            <!-- Context log notes if any -->
            <div v-if="ev.contextNotes" class="text-[11px] font-mono text-gray-500 dark:text-slate-400 bg-gray-50 dark:bg-slate-950/80 px-2.5 py-1 rounded-lg border border-gray-100 dark:border-slate-800 truncate">
              {{ ev.contextNotes }}
            </div>
          </div>

          <!-- Technical Metrics right -->
          <div class="text-right flex-shrink-0 font-mono text-xs space-y-1 text-gray-500 dark:text-slate-400">
            <div>CPU: <strong class="text-gray-800 dark:text-slate-200">{{ ev.cpuTimeMs }} ms</strong></div>
            <div>GPU: <strong class="text-gray-800 dark:text-slate-200">{{ ev.gpuTimeMs }} ms</strong></div>
            <div v-if="ev.vramUsagePercent">VRAM: <strong class="text-purple-600 dark:text-purple-400">{{ ev.vramUsagePercent }}%</strong></div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { useAppStore } from '@/stores/app'
import { useToastStore } from '@/stores/toast'
import type { StutterEvent, StutterMonitorStatus } from '@/types'

const { t, te } = useI18n()
const appStore = useAppStore()
const toastStore = useToastStore()

const status = ref<StutterMonitorStatus | null>(null)
const events = ref<StutterEvent[]>([])
const selectedThreshold = ref(15)
const showSetupGuide = ref(false)
const showSimMenu = ref(false)

// Waveform history buffer (last 50 points)
const waveformHistory = ref<number[]>(Array(50).fill(55))

let pollTimer: ReturnType<typeof setInterval> | null = null

const cpuPercent = computed(() => {
  if (!status.value || !status.value.isConnected) return 50
  const cpu = status.value.cpuTimeMs
  const gpu = status.value.gpuTimeMs
  const total = cpu + gpu
  if (total <= 0) return 50
  return Math.min(90, Math.max(10, Math.round((cpu / total) * 100)))
})

const bottleneckHint = computed(() => {
  if (!status.value || !status.value.isConnected) return t('stutterDetector.noDataHint')
  const cpu = status.value.cpuTimeMs
  const gpu = status.value.gpuTimeMs
  if (cpu > gpu * 1.35) return t('stutterDetector.cpuHeavier')
  if (gpu > cpu * 1.35) return t('stutterDetector.gpuHeavier')
  return t('stutterDetector.balanced')
})

const lowestFpsRecorded = computed(() => {
  if (events.value.length === 0) return null
  return Math.min(...events.value.map((e) => e.lowestFps))
})

// SVG Waveform Coordinates
const maxDisplayFps = 80
const thresholdY = computed(() => {
  const norm = Math.max(0, Math.min(maxDisplayFps, selectedThreshold.value))
  return (1 - norm / maxDisplayFps) * 100
})

const latestPointX = computed(() => 500)
const latestPointY = computed(() => {
  const lastVal = waveformHistory.value[waveformHistory.value.length - 1] || 0
  const norm = Math.max(0, Math.min(maxDisplayFps, lastVal))
  return (1 - norm / maxDisplayFps) * 100
})

const chartLinePath = computed(() => {
  const points = waveformHistory.value
  if (!points || points.length === 0) return ''
  const stepX = 500 / (points.length - 1)

  const coords = points.map((fps, i) => {
    const norm = Math.max(0, Math.min(maxDisplayFps, fps))
    const y = (1 - norm / maxDisplayFps) * 100
    return { x: i * stepX, y }
  })

  // Smooth bezier curve
  let path = `M ${coords[0].x} ${coords[0].y}`
  for (let i = 0; i < coords.length - 1; i++) {
    const current = coords[i]
    const next = coords[i + 1]
    const cpX = (current.x + next.x) / 2
    path += ` C ${cpX} ${current.y}, ${cpX} ${next.y}, ${next.x} ${next.y}`
  }
  return path
})

const chartAreaPath = computed(() => {
  const line = chartLinePath.value
  if (!line) return ''
  return `${line} L 500 100 L 0 100 Z`
})

function causeBadgeClass(cause: string): string {
  switch (cause) {
    case 'vram_overload':
      return 'bg-purple-100 text-purple-800 dark:bg-purple-950/60 dark:text-purple-300 border border-purple-200 dark:border-purple-800/60'
    case 'cpu_bottleneck':
      return 'bg-blue-100 text-blue-800 dark:bg-blue-950/60 dark:text-blue-300 border border-blue-200 dark:border-blue-800/60'
    case 'gpu_overload':
      return 'bg-red-100 text-red-800 dark:bg-red-950/60 dark:text-red-300 border border-red-200 dark:border-red-900/60'
    case 'scenery_disk_io':
      return 'bg-amber-100 text-amber-800 dark:bg-amber-950/60 dark:text-amber-300 border border-amber-200 dark:border-amber-800/60'
    default:
      return 'bg-gray-100 text-gray-800 dark:bg-slate-800/60 dark:text-slate-300 border border-gray-200 dark:border-slate-800'
  }
}

function getEventTitle(ev: StutterEvent): string {
  const key = `stutterDetector.causes.${ev.primaryCause}.title`
  return te(key) ? t(key) : ev.causeTitle
}

function getEventDesc(ev: StutterEvent): string {
  const key = `stutterDetector.causes.${ev.primaryCause}.desc`
  return te(key) ? t(key) : ev.causeDescription
}

function getEventSugg(ev: StutterEvent): string {
  const key = `stutterDetector.causes.${ev.primaryCause}.sugg`
  return te(key) ? t(key) : ev.suggestion
}

async function loadData() {
  try {
    const [st, evs] = await Promise.all([
      invoke<StutterMonitorStatus>('get_stutter_monitor_status'),
      invoke<StutterEvent[]>('get_stutter_events')
    ])
    status.value = st
    events.value = evs
    if (st.thresholdFps) {
      selectedThreshold.value = Math.round(st.thresholdFps)
    }

    // Update real-time waveform buffer
    if (st && st.isConnected && st.currentFps > 0) {
      waveformHistory.value.push(st.currentFps)
      if (waveformHistory.value.length > 50) {
        waveformHistory.value.shift()
      }
    }
  } catch (err) {
    console.error('Failed to load stutter status', err)
  }
}

// Background lightweight polling when window is minimized (HWiNFO style)
async function loadDataMinimal() {
  try {
    const st = await invoke<StutterMonitorStatus>('get_stutter_monitor_status')
    status.value = st
  } catch (err) {
    console.error('Failed to poll stutter status in background', err)
  }
}

function handleVisibilityChange() {
  if (document.hidden) {
    // Window is hidden / minimized to system tray: completely freeze polling to use 0.0% CPU & GPU!
    // Rust UDP detector in background keeps silently recording stutters with 0% overhead.
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
  } else {
    // Window restored from tray: immediately fetch all accumulated stutters and resume 1000ms polling
    loadData()
    if (pollTimer) {
      clearInterval(pollTimer)
      pollTimer = null
    }
    pollTimer = setInterval(loadData, 1000)
  }
}

async function handleToggleMonitor() {
  if (status.value?.isListening) {
    try {
      await invoke('stop_stutter_monitor')
      toastStore.info(t('stutterDetector.monitorStopped'))
      await loadData()
    } catch (err: any) {
      toastStore.error(err?.message || 'Error stopping monitor')
    }
  } else {
    try {
      await invoke('start_stutter_monitor', {
        port: 49005,
        threshold: selectedThreshold.value,
        xplanePath: appStore.xplanePath
      })
      toastStore.success(t('stutterDetector.monitorStarted'))
      await loadData()
    } catch (err: any) {
      toastStore.error(err?.message || 'Error starting monitor')
    }
  }
}

async function handleThresholdChange() {
  try {
    await invoke('start_stutter_monitor', {
      port: status.value?.port || 49005,
      threshold: selectedThreshold.value,
      xplanePath: appStore.xplanePath
    })
    toastStore.info(t('stutterDetector.thresholdUpdated', { fps: selectedThreshold.value }))
    await loadData()
  } catch (err) {
    console.error(err)
  }
}

async function triggerSimulate(scenario: string) {
  showSimMenu.value = false
  try {
    const ev = await invoke<StutterEvent>('simulate_stutter_event', { scenario })
    events.value.unshift(ev)
    // Add simulated dip to waveform
    waveformHistory.value.push(ev.lowestFps)
    if (waveformHistory.value.length > 50) {
      waveformHistory.value.shift()
    }
    const translatedTitle = getEventTitle(ev)
    toastStore.warning(`Simulated drop: ${ev.lowestFps} FPS (${translatedTitle})`)
    await loadData()
  } catch (err: any) {
    toastStore.error(err?.message || 'Simulation error')
  }
}

async function handleClearEvents() {
  try {
    await invoke('clear_stutter_events')
    events.value = []
    toastStore.info(t('stutterDetector.historyCleared'))
    await loadData()
  } catch (err: any) {
    toastStore.error(err?.message || 'Clear failed')
  }
}

onMounted(async () => {
  // Automatically start listener if not already running
  try {
    await invoke('start_stutter_monitor', {
      port: 49005,
      threshold: selectedThreshold.value,
      xplanePath: appStore.xplanePath
    })
  } catch (e) {
    console.error(e)
  }

  await loadData()
  pollTimer = setInterval(loadData, 1000)
  document.addEventListener('visibilitychange', handleVisibilityChange)
})

onUnmounted(() => {
  if (pollTimer) {
    clearInterval(pollTimer)
    pollTimer = null
  }
  document.removeEventListener('visibilitychange', handleVisibilityChange)
})
</script>
