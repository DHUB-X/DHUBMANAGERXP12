<template>
  <div
    class="home-view h-full flex flex-col p-4 sm:p-5 lg:p-6 relative overflow-y-auto overflow-x-hidden select-none"
  >
    <!-- Background Decor (Static High-Performance Ambient Glow - 0% GPU load) -->
    <div
      class="absolute inset-0 pointer-events-none z-0 opacity-40 dark:opacity-100 transition-opacity duration-300"
      style="background: radial-gradient(circle at 15% 15%, rgba(56, 189, 248, 0.08) 0%, transparent 45%), radial-gradient(circle at 85% 85%, rgba(37, 99, 235, 0.08) 0%, transparent 45%);"
    ></div>

    <div class="w-full z-10 flex flex-col flex-1 min-h-0 gap-2.5 sm:gap-3">
      <!-- Warning Alert (Compact) -->
      <transition name="slide-down">
        <div
          v-if="!store.xplanePath"
          class="flex-shrink-0 bg-yellow-50/90 dark:bg-yellow-500/10 border border-yellow-200 dark:border-yellow-500/20 rounded-xl p-3 flex items-center space-x-3 shadow-lg shadow-yellow-500/5 transition-colors duration-300"
        >
          <div class="p-2 bg-yellow-100 dark:bg-yellow-500/20 rounded-lg flex-shrink-0">
            <svg
              class="w-5 h-5 text-yellow-600 dark:text-yellow-400"
              fill="none"
              stroke="currentColor"
              viewBox="0 0 24 24"
            >
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
              ></path>
            </svg>
          </div>
          <div class="flex-1 min-w-0">
            <p class="text-sm font-medium text-yellow-800 dark:text-yellow-100 truncate">
              <AnimatedText>{{ $t('home.setPathFirst') }}</AnimatedText>
            </p>
            <p class="text-xs text-yellow-700 dark:text-yellow-200/70 truncate">
              <AnimatedText>{{ $t('home.pathNotSetDesc') }}</AnimatedText>
            </p>
          </div>
          <router-link
            to="/settings"
            class="flex-shrink-0 inline-flex items-center px-3 py-1.5 bg-yellow-200/50 dark:bg-yellow-500/20 hover:bg-yellow-200 dark:hover:bg-yellow-500/30 text-yellow-800 dark:text-yellow-200 text-xs font-medium rounded-lg transition-colors duration-200 border border-yellow-300 dark:border-yellow-500/30"
          >
            <AnimatedText>{{ $t('home.goToSettings') }}</AnimatedText>
            <svg class="w-3 h-3 ml-1" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path
                stroke-linecap="round"
                stroke-linejoin="round"
                stroke-width="2"
                d="M9 5l7 7-7 7"
              ></path>
            </svg>
          </router-link>
        </div>
      </transition>

      <!-- Executive Flight Deck Hero Card (KAYAN Inspired Layout - 0% GPU overhead) -->
      <div
        v-if="store.xplanePath && !store.isAnalyzing && !store.isInstalling && !store.showCompletion"
        class="bg-white/90 dark:bg-slate-900/90 text-slate-800 dark:text-white rounded-2xl p-3.5 sm:p-4 border border-slate-200/80 dark:border-sky-500/20 shadow-lg dark:shadow-xl relative overflow-hidden shrink-0 transition-colors"
      >
        <div class="relative z-10 flex flex-col lg:flex-row lg:items-center justify-between gap-3 sm:gap-4">
          <!-- Left: Welcome & Flight Deck Identification -->
          <div class="space-y-1 max-w-xl">
            <!-- Top Status Pill -->
            <div class="flex items-center gap-2">
              <div
                class="px-2.5 py-0.5 rounded-full text-xs font-semibold flex items-center gap-1.5 transition-colors border"
                :class="isXPlaneRunning 
                  ? 'bg-emerald-500/15 border-emerald-400/30 text-emerald-600 dark:text-emerald-300'
                  : 'bg-sky-500/15 border-sky-400/30 text-sky-600 dark:text-sky-300'"
              >
                <span
                  class="w-2 h-2 rounded-full shrink-0"
                  :class="isXPlaneRunning ? 'bg-emerald-400 shadow-[0_0_8px_rgba(52,211,153,0.8)]' : 'bg-sky-400 shadow-[0_0_8px_rgba(56,189,248,0.8)]'"
                ></span>
                <span>{{ isXPlaneRunning ? $t('home.xplaneRunning') : $t('home.simulatorReady') }}</span>
              </div>
              <span class="text-[11px] text-slate-500 dark:text-slate-400 font-mono tracking-wide">
                {{ xpVersionName }} Ecosystem
              </span>
            </div>

            <!-- Main Heading -->
            <h1 class="text-lg sm:text-xl font-extrabold tracking-tight text-slate-900 dark:text-white flex items-center gap-2">
              <span>{{ $t('home.flightDeckOverview') }}</span>
            </h1>
            <p class="text-[11px] sm:text-xs text-slate-500 dark:text-slate-400 font-normal leading-relaxed">
              {{ $t('home.flightDeckSubtitle') }}
            </p>
          </div>

          <!-- Right: Horizontal Metric Capsules -->
          <div class="flex items-center gap-2 sm:gap-2.5 shrink-0 flex-wrap sm:flex-nowrap">
            <!-- Fleet Metric Widget -->
            <router-link
              to="/management?tab=aircraft"
              class="flex items-center gap-2.5 px-3 py-2 rounded-xl bg-slate-50 hover:bg-slate-100 dark:bg-white/5 dark:hover:bg-white/10 border border-slate-200 dark:border-white/10 hover:border-sky-400/40 transition-all duration-200 group cursor-pointer"
            >
              <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-full bg-sky-500/15 border border-sky-400/30 flex items-center justify-center text-sky-400 group-hover:scale-105 transition-transform shadow-inner shrink-0">
                <svg class="w-4 h-4 sm:w-4.5 sm:h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
                </svg>
              </div>
              <div>
                <div class="text-lg sm:text-xl font-extrabold tracking-tight text-slate-900 dark:text-white group-hover:text-sky-300 transition-colors leading-tight">
                  {{ managementStore.aircraft.length || 0 }}
                </div>
                <div class="text-[9px] sm:text-[10px] font-semibold text-slate-500 dark:text-slate-400 tracking-wider uppercase">
                  Fleet Aircraft
                </div>
              </div>
            </router-link>

            <!-- Plugins Metric Widget -->
            <router-link
              to="/management?tab=plugin"
              class="flex items-center gap-2.5 px-3 py-2 rounded-xl bg-slate-50 hover:bg-slate-100 dark:bg-white/5 dark:hover:bg-white/10 border border-slate-200 dark:border-white/10 hover:border-emerald-400/40 transition-all duration-200 group cursor-pointer"
            >
              <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-full bg-emerald-500/15 border border-emerald-400/30 flex items-center justify-center text-emerald-400 group-hover:scale-105 transition-transform shadow-inner shrink-0">
                <svg class="w-4 h-4 sm:w-4.5 sm:h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M11 4a2 2 0 114 0v1a1 1 0 001 1h3a1 1 0 011 1v3a1 1 0 01-1 1h-1a2 2 0 100 4h1a1 1 0 011 1v3a1 1 0 01-1 1h-3a1 1 0 01-1-1v-1a2 2 0 10-4 0v1a1 1 0 01-1 1H7a1 1 0 01-1-1v-3a1 1 0 00-1-1H4a2 2 0 110-4h1a1 1 0 001-1V7a1 1 0 011-1h3a1 1 0 001-1V4z" />
                </svg>
              </div>
              <div>
                <div class="text-lg sm:text-xl font-extrabold tracking-tight text-slate-900 dark:text-white group-hover:text-emerald-300 transition-colors leading-tight">
                  {{ managementStore.plugins.length || 0 }}
                </div>
                <div class="text-[9px] sm:text-[10px] font-semibold text-slate-500 dark:text-slate-400 tracking-wider uppercase">
                  Active Plugins
                </div>
              </div>
            </router-link>

            <!-- Scenery Metric Widget -->
            <router-link
              to="/scenery"
              class="flex items-center gap-2.5 px-3 py-2 rounded-xl bg-slate-50 hover:bg-slate-100 dark:bg-white/5 dark:hover:bg-white/10 border border-slate-200 dark:border-white/10 hover:border-blue-400/40 transition-all duration-200 group cursor-pointer"
            >
              <div class="w-8 h-8 sm:w-9 sm:h-9 rounded-full bg-blue-500/15 border border-blue-400/30 flex items-center justify-center text-blue-400 group-hover:scale-105 transition-transform shadow-inner shrink-0">
                <svg class="w-4 h-4 sm:w-4.5 sm:h-4.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3.055 11H5a2 2 0 012 2v1a2 2 0 002 2 2 2 0 012 2v2.945M8 3.935V5.5A2.5 2.5 0 0010.5 8h.5a2 2 0 012 2 2 2 0 104 0 2 2 0 012-2h1.064M15 20.488V18a2 2 0 012-2h3.064M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
              </div>
              <div>
                <div class="text-lg sm:text-xl font-extrabold tracking-tight text-slate-900 dark:text-white group-hover:text-blue-300 transition-colors leading-tight">
                  {{ sceneryStore.entries.length || 0 }}
                </div>
                <div class="text-[9px] sm:text-[10px] font-semibold text-slate-500 dark:text-slate-400 tracking-wider uppercase">
                  Scenery Packs
                </div>
              </div>
            </router-link>
          </div>
        </div>
      </div>

      <!-- Main Action Depot Card (0% GPU, responsive padding to prevent clipping) -->
      <div
        class="flex-1 min-h-0 bg-white/90 dark:bg-slate-900/80 text-slate-800 dark:text-white rounded-2xl p-3.5 sm:p-4.5 border border-slate-200/80 dark:border-sky-500/20 shadow-lg dark:shadow-xl relative flex flex-col transition-colors"
      >
        <!-- Card Header -->
        <div class="flex items-center justify-between shrink-0 mb-2 pb-2 border-b border-slate-200/60 dark:border-white/5">
          <div>
            <h2 class="text-sm font-bold tracking-tight text-slate-900 dark:text-white flex items-center gap-1.5">
              <svg class="w-4 h-4 text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12" />
              </svg>
              <span>Addon Installation Depot</span>
            </h2>
            <p class="text-[10.5px] text-slate-500 dark:text-slate-400 mt-0.5">
              Drag and drop add-ons or click anywhere to browse archives and scenery packages.
            </p>
          </div>
          <router-link
            to="/scenery"
            class="text-xs text-sky-600 dark:text-sky-400 hover:text-sky-500 dark:hover:text-sky-300 font-medium flex items-center gap-1 transition-colors"
          >
            <span>Manage scenery</span>
            <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
            </svg>
          </router-link>
        </div>

        <!-- Inner Action Area: Toggle between Drop Zone and Progress Overlays -->
        <div
          class="flex-1 min-h-0 bg-sky-50/50 dark:bg-slate-900/50 border-2 border-dashed border-sky-300/60 dark:border-sky-500/20 rounded-xl p-3 sm:p-4 text-center transition-all duration-200 hover:border-sky-400 dark:hover:border-sky-400/60 hover:bg-sky-500/5 flex flex-col items-center justify-center relative drop-zone-card group overflow-y-auto"
          :class="{
            'drag-over ring-4 ring-sky-500/20 border-sky-500 scale-[1.01]': isDragging,
            'animate-pulse border-sky-400': store.isAnalyzing,
            'debug-drop': debugDropFlash,
          }"
          @click="handleDropZoneClick"
        >
          <!-- Subtle Aerospace Corner Markers -->
          <div class="absolute top-2.5 left-2.5 w-2.5 h-2.5 border-t-2 border-l-2 border-sky-400/50 dark:border-sky-400/30 rounded-tl-sm pointer-events-none transition-colors group-hover:border-sky-500 dark:group-hover:border-sky-400/70"></div>
          <div class="absolute top-2.5 right-2.5 w-2.5 h-2.5 border-t-2 border-r-2 border-sky-400/50 dark:border-sky-400/30 rounded-tr-sm pointer-events-none transition-colors group-hover:border-sky-500 dark:group-hover:border-sky-400/70"></div>
          <div class="absolute bottom-2.5 left-2.5 w-2.5 h-2.5 border-b-2 border-l-2 border-sky-400/50 dark:border-sky-400/30 rounded-bl-sm pointer-events-none transition-colors group-hover:border-sky-500 dark:group-hover:border-sky-400/70"></div>
          <div class="absolute bottom-2.5 right-2.5 w-2.5 h-2.5 border-b-2 border-r-2 border-sky-400/50 dark:border-sky-400/30 rounded-br-sm pointer-events-none transition-colors group-hover:border-sky-500 dark:group-hover:border-sky-400/70"></div>

          <!-- Main Area Content: Toggle between Drop Zone and Progress Overlays -->
          <transition name="fade" mode="out-in">
            <!-- Drop Zone State -->
            <div
              v-if="!store.isAnalyzing && !store.isInstalling && !store.showCompletion"
              key="drop-zone"
              class="relative z-10 flex flex-col items-center justify-center space-y-2 sm:space-y-2.5 max-w-xl mx-auto my-auto py-1"
            >
              <!-- Modern Squircle Icon Container -->
              <div
                class="w-11 h-11 sm:w-12 sm:h-12 rounded-xl bg-gradient-to-br from-sky-500/10 to-blue-600/10 dark:from-sky-500/20 dark:to-blue-600/15 border border-sky-400/20 dark:border-sky-400/25 flex items-center justify-center group-hover:scale-105 transition-all duration-200 relative shadow-sm"
              >
                <svg
                  class="w-5 h-5 sm:w-6 sm:h-6 text-sky-500 dark:text-sky-400 transition-transform duration-200 group-hover:translate-y-[-2px]"
                  :class="{ 'animate-bounce': isDragging }"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="1.8"
                    d="M7 16a4 4 0 01-.88-7.903A5 5 0 1115.9 6L16 6a5 5 0 011 9.9M15 13l-3-3m0 0l-3 3m3-3v12"
                  ></path>
                </svg>
              </div>

              <!-- Text Content -->
              <div class="space-y-0.5 text-center px-4">
                <h2
                  class="text-lg sm:text-xl font-extrabold text-slate-900 dark:text-white tracking-tight group-hover:text-sky-600 dark:group-hover:text-sky-300 transition-colors"
                >
                  <AnimatedText>{{ $t('home.dropFilesHere') }}</AnimatedText>
                </h2>
                <p class="text-slate-500 dark:text-slate-400 max-w-md mx-auto text-xs leading-relaxed font-normal">
                  <AnimatedText>{{ $t('home.supportedFormats') }}</AnimatedText>
                </p>
              </div>

              <!-- Modern Feature Badges -->
              <div class="flex flex-wrap justify-center gap-1.5 pt-0.5">
                <span
                  class="px-2.5 py-1 rounded-lg bg-white dark:bg-white/5 border border-slate-200 dark:border-white/10 text-slate-700 dark:text-slate-300 text-[11px] font-medium flex items-center space-x-1.5 shadow-xs transition hover:border-sky-400/70"
                >
                  <svg
                    class="w-3 h-3 text-sky-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z"
                    ></path>
                  </svg>
                  <span><AnimatedText>{{ $t('home.autoDetect') }}</AnimatedText></span>
                </span>
                <span
                  class="px-2.5 py-1 rounded-lg bg-white dark:bg-white/5 border border-slate-200 dark:border-white/10 text-slate-700 dark:text-slate-300 text-[11px] font-medium flex items-center space-x-1.5 shadow-xs transition hover:border-sky-400/70"
                >
                  <svg
                    class="w-3 h-3 text-sky-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M13 10V3L4 14h7v7l9-11h-7z"
                    ></path>
                  </svg>
                  <span><AnimatedText>{{ $t('home.fastInstall') }}</AnimatedText></span>
                </span>
                <span
                  class="px-2.5 py-1 rounded-lg bg-white dark:bg-white/5 border border-slate-200 dark:border-white/10 text-slate-700 dark:text-slate-300 text-[11px] font-medium flex items-center space-x-1.5 shadow-xs transition hover:border-sky-400/70"
                >
                  <svg
                    class="w-3 h-3 text-blue-400"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10"
                    ></path>
                  </svg>
                  <span><AnimatedText>{{ $t('home.batchProcess') }}</AnimatedText></span>
                </span>
              </div>

              <!-- Action Buttons (Clear, unclipped CTAs) -->
              <div class="flex items-center gap-2.5 pt-1">
                <button
                  type="button"
                  class="px-3.5 py-1.5 bg-white hover:bg-slate-50 dark:bg-white/10 dark:hover:bg-white/15 text-slate-700 dark:text-white font-medium text-xs rounded-xl border border-slate-200 dark:border-white/15 hover:border-sky-400/80 transition-all duration-200 flex items-center gap-1.5 cursor-pointer shadow-xs active:scale-95"
                  @click.stop="handleDropZoneClick"
                >
                  <svg class="w-3.5 h-3.5 text-sky-500 dark:text-sky-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4v16m8-8H4" />
                  </svg>
                  <span>Browse Addons</span>
                </button>

                <button
                  v-if="store.xplanePath"
                  :disabled="isLaunchingXPlane || isXPlaneRunning"
                  :title="isXPlaneRunning ? $t('home.xplaneAlreadyRunning') : ''"
                  class="px-4 py-1.5 bg-gradient-to-r from-sky-500 to-blue-600 hover:from-sky-400 hover:to-blue-500 text-white font-medium text-xs rounded-xl transition-all duration-200 hover:scale-[1.02] active:scale-[0.98] flex items-center gap-1.5 shadow-md shadow-sky-500/20 hover:shadow-sky-500/30 disabled:opacity-60 disabled:cursor-not-allowed disabled:hover:scale-100 disabled:shadow-none"
                  @click.stop="handleLaunchXPlane"
                >
                  <svg
                    v-if="isLaunchingXPlane"
                    class="w-3.5 h-3.5 animate-spin"
                    fill="none"
                    viewBox="0 0 24 24"
                  >
                    <circle
                      class="opacity-25"
                      cx="12"
                      cy="12"
                      r="10"
                      stroke="currentColor"
                      stroke-width="4"
                    ></circle>
                    <path
                      class="opacity-75"
                      fill="currentColor"
                      d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                    ></path>
                  </svg>
                  <svg
                    v-else
                    class="w-3.5 h-3.5"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M14.752 11.168l-3.197-2.132A1 1 0 0010 9.87v4.263a1 1 0 001.555.832l3.197-2.132a1 1 0 000-1.664z"
                    ></path>
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                    ></path>
                  </svg>
                  <span><AnimatedText>{{ $t('home.launchXPlane') }}</AnimatedText></span>
                </button>
              </div>
            </div>

            <!-- Analyzing State -->
            <AnalyzingOverlay v-else-if="store.isAnalyzing" key="analyzing" />

            <!-- Installing / Completion State -->
            <InstallProgressOverlay
              v-else-if="store.isInstalling || store.showCompletion"
              key="installing"
              :percentage="progressStore.formatted.percentage"
              :task-name="progressStore.formatted.taskName"
              :processed-m-b="progressStore.formatted.processedMB"
              :total-m-b="progressStore.formatted.totalMB"
              :task-progress="progressStore.formatted.taskProgress"
              :tasks="store.installingTasks"
              :current-task-index="progressStore.progress?.currentTaskIndex ?? 0"
              :current-task-percentage="progressStore.formatted.currentTaskPercentage"
              :current-task-processed-m-b="progressStore.formatted.currentTaskProcessedMB"
              :current-task-total-m-b="progressStore.formatted.currentTaskTotalMB"
              :is-complete="store.showCompletion"
              :install-result="store.installResult"
              :active-tasks="progressStore.activeTasks"
              :completed-task-count="progressStore.completedTaskCount"
              :completed-task-ids="progressStore.completedTaskIds"
              @skip="handleSkipTask"
              @cancel="handleCancelInstallation"
              @confirm="handleCompletionConfirm"
            />
          </transition>
        </div>
      </div>

      <ConfirmationModal
        v-if="showConfirmation"
        @close="showConfirmation = false"
        @confirm="handleInstall"
      />
      <PasswordModal
        v-if="showPasswordModal"
        :archive-paths="passwordRequiredPaths"
        :error-message="passwordErrorMessage"
        @confirm="handlePasswordSubmit"
        @cancel="handlePasswordCancel"
      />

      <!-- Launch X-Plane Confirmation Dialog -->
      <transition name="fade">
        <div
          v-if="showLaunchConfirmDialog"
          class="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm"
          @click.self="cancelLaunchDialog"
        >
          <div
            class="bg-white dark:bg-gray-800 rounded-2xl shadow-2xl max-w-md w-full mx-4 overflow-hidden animate-bounce-in"
          >
            <!-- Header -->
            <div
              class="px-6 py-4 border-b border-gray-200 dark:border-gray-700 flex items-center gap-3"
            >
              <div class="p-2 bg-amber-100 dark:bg-amber-500/20 rounded-lg">
                <svg
                  class="w-5 h-5 text-amber-600 dark:text-amber-400"
                  fill="none"
                  stroke="currentColor"
                  viewBox="0 0 24 24"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    stroke-width="2"
                    d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z"
                  ></path>
                </svg>
              </div>
              <h3 class="text-lg font-semibold text-gray-900 dark:text-white">
                {{ $t('home.launchConfirmTitle') }}
              </h3>
            </div>

            <!-- Content -->
            <div class="px-6 py-4">
              <p class="text-gray-600 dark:text-gray-300">
                {{ $t('home.launchConfirmMessage') }}
              </p>
            </div>

            <!-- Actions -->
            <div
              class="px-6 py-4 bg-gray-50 dark:bg-gray-800/50 flex flex-col sm:flex-row gap-3 sm:justify-end"
            >
              <button
                :disabled="isLaunchingXPlane"
                class="px-4 py-2 text-gray-700 dark:text-gray-300 bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                @click="cancelLaunchDialog"
              >
                {{ $t('common.cancel') }}
              </button>
              <button
                :disabled="isLaunchingXPlane"
                class="px-4 py-2 text-blue-700 dark:text-blue-300 bg-blue-100 dark:bg-blue-500/20 hover:bg-blue-200 dark:hover:bg-blue-500/30 rounded-lg font-medium transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                @click="launchXPlane"
              >
                <svg
                  v-if="isLaunchingXPlane"
                  class="w-4 h-4 animate-spin"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                  ></circle>
                  <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  ></path>
                </svg>
                {{ $t('home.launchDirectly') }}
              </button>
              <button
                :disabled="isLaunchingXPlane"
                class="px-4 py-2 text-white bg-gradient-to-r from-blue-500 to-blue-600 hover:from-blue-600 hover:to-blue-700 rounded-lg font-medium transition-all shadow-md disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
                @click="applyAndLaunch"
              >
                <svg
                  v-if="isLaunchingXPlane"
                  class="w-4 h-4 animate-spin"
                  fill="none"
                  viewBox="0 0 24 24"
                >
                  <circle
                    class="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    stroke-width="4"
                  ></circle>
                  <path
                    class="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  ></path>
                </svg>
                {{ $t('home.applyAndLaunch') }}
              </button>
            </div>
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, watch } from 'vue'
import { useAppStore } from '@/stores/app'
import { useToastStore } from '@/stores/toast'
import { useModalStore } from '@/stores/modal'
import { useProgressStore } from '@/stores/progress'
import { useUpdateStore } from '@/stores/update'
import { useSceneryStore } from '@/stores/scenery'
import { useManagementStore } from '@/stores/management'
import { useLockStore } from '@/stores/lock'
import { useI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow'
import { listen } from '@tauri-apps/api/event'
import type { UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import ConfirmationModal from '@/components/ConfirmationModal.vue'
import PasswordModal from '@/components/PasswordModal.vue'
import AnimatedText from '@/components/AnimatedText.vue'
import InstallProgressOverlay from '@/components/InstallProgressOverlay.vue'
import AnalyzingOverlay from '@/components/AnalyzingOverlay.vue'
import type { AnalysisResult, InstallProgress, InstallResult } from '@/types'
import { AddonType } from '@/types'
import { getErrorMessage } from '@/types'
import { logOperation, logError, logDebug, logBasic } from '@/services/logger'
import { setTrackedTimeout } from '@/utils/timeout'

const { t } = useI18n()

const store = useAppStore()
const toast = useToastStore()
const modal = useModalStore()
const updateStore = useUpdateStore()
const progressStore = useProgressStore()
const sceneryStore = useSceneryStore()
const managementStore = useManagementStore()
const lockStore = useLockStore()
const isDragging = ref(false)
const showConfirmation = ref(false)
const showLaunchConfirmDialog = ref(false)
const isLaunchingXPlane = ref(false)
const isXPlaneRunning = ref(false)
const xpVersionName = 'X-Plane 12'
let xplaneCheckInterval: number | null = null
const debugDropFlash = ref(false)

// Sync confirmation modal state with store for exit confirmation
watch(showConfirmation, (value) => {
  store.setConfirmationOpen(value)
})

// Password modal state
const showPasswordModal = ref(false)
const passwordRequiredPaths = ref<string[]>([])
const pendingAnalysisPaths = ref<string[]>([])
const collectedPasswords = ref<Record<string, string>>({})
const passwordRetryCount = ref(0)
const passwordErrorMessage = ref('')
const MAX_PASSWORD_RETRIES = 3

// Password rate limiting
const passwordAttemptTimestamps = ref<number[]>([])
const MIN_PASSWORD_ATTEMPT_DELAY_MS = 1000 // 1 second between attempts
const PASSWORD_RATE_LIMIT_WINDOW_MS = 10000 // 10 second window for rate limiting
const DEBUG_DROP_FLASH_DURATION_MS = 800 // Duration for debug drop flash visual feedback
const DROP_ZONE_CLICK_SUPPRESS_AFTER_FOCUS_MS = 350 // Prevent activation click from opening picker
const COMPLETION_ANIMATION_DELAY_MS = 100 // Brief delay before hiding progress to allow animation to start
const suppressDropZoneClickUntil = ref(0)
const windowWasBlurred = ref(!document.hasFocus())

// Timer tracking for cleanup on unmount to prevent memory leaks
// Using Set for O(1) add/delete operations instead of array
const activeTimeoutIds = new Set<ReturnType<typeof setTimeout>>()

// Tauri drag-drop event unsubscribe function
let unlistenDragDrop: UnlistenFn | null = null
let unlistenProgress: UnlistenFn | null = null
let unlistenDeletionSkipped: UnlistenFn | null = null

// Watch for pending CLI args changes
watch(
  () => store.pendingCliArgs,
  async (args) => {
    if (args && args.length > 0) {
      // Use store mutex flag for TOCTOU-safe concurrency control
      // This prevents race conditions when multiple watch events fire quickly
      if (store.isAnalyzeInProgress || store.isAnalyzing || store.isInstalling) {
        logDebug('Analysis in progress, re-queueing args for later', 'app')
        store.addCliArgsToBatch(args)
        store.clearPendingCliArgs()
        return
      }

      // Set mutex immediately before any async operation
      store.isAnalyzeInProgress = true

      logDebug(`Processing pending CLI args from watcher: ${args.join(', ')}`, 'app')
      const argsCopy = [...args]
      store.clearPendingCliArgs()
      try {
        await analyzeFiles(argsCopy)
      } catch (error) {
        logError(`Failed to process CLI args: ${error}`, 'app')
        modal.showError(getErrorMessage(error))
      } finally {
        // Always release mutex
        store.isAnalyzeInProgress = false
      }
    }
  },
)

// Global listeners for drag/drop visual feedback
function onWindowDragOver(e: DragEvent) {
  e.preventDefault()
  // Ignore drag events when installing
  if (store.isInstalling) {
    return
  }
  isDragging.value = true
}

function onWindowDragLeave(e: DragEvent) {
  // Ignore drag events when installing
  if (store.isInstalling) {
    return
  }
  // Only set to false if leaving the window
  if (!e.relatedTarget) {
    isDragging.value = false
  }
}

function onWindowDrop(e: DragEvent) {
  logDebug('Window drop event (HTML5)', 'drag-drop')
  e.preventDefault()
  // Ignore drop events when installing
  if (store.isInstalling) {
    return
  }
  isDragging.value = false
  debugDropFlash.value = true
  setTrackedTimeout(
    () => (debugDropFlash.value = false),
    DEBUG_DROP_FLASH_DURATION_MS,
    activeTimeoutIds,
  )
}

function onWindowFocus() {
  if (windowWasBlurred.value) {
    suppressDropZoneClickUntil.value = Date.now() + DROP_ZONE_CLICK_SUPPRESS_AFTER_FOCUS_MS
  }
  windowWasBlurred.value = false
}

function onWindowBlur() {
  windowWasBlurred.value = true
  suppressDropZoneClickUntil.value = 0
}

async function handleDropZoneClick() {
  if (store.isInstalling || store.isAnalyzing || store.showCompletion) {
    return
  }

  if (!document.hasFocus() || Date.now() < suppressDropZoneClickUntil.value) {
    logDebug('Ignoring drop-zone click while window is inactive/just activated', 'drag-drop')
    return
  }

  try {
    const selected = await open({
      multiple: true,
      title: t('home.dropFilesHere'),
      filters: [
        {
          name: 'Addon Files',
          extensions: ['zip', '7z', 'rar', 'lua'],
        },
        {
          name: 'All Files',
          extensions: ['*'],
        },
      ],
    })

    const paths = Array.isArray(selected) ? selected : selected ? [selected] : []
    if (paths.length === 0) {
      return
    }

    await analyzeFiles(paths)
  } catch (error) {
    logError(`Failed to open file picker: ${error}`, 'drag-drop')
    modal.showError(getErrorMessage(error))
  }
}

onMounted(async () => {
  window.addEventListener('dragover', onWindowDragOver)
  window.addEventListener('dragleave', onWindowDragLeave)
  window.addEventListener('drop', onWindowDrop)
  window.addEventListener('focus', onWindowFocus)
  window.addEventListener('blur', onWindowBlur)

  // Use Tauri 2's native drag-drop event for getting file paths
  try {
    const webview = getCurrentWebviewWindow()
    unlistenDragDrop = await webview.onDragDropEvent(async (event) => {
      logDebug(`Tauri drag-drop event: ${event.payload.type}`, 'drag-drop')

      // Ignore all drag-drop events when installing
      if (store.isInstalling) {
        logDebug('Ignoring drag-drop event (installing)', 'drag-drop')
        return
      }

      if (event.payload.type === 'over') {
        isDragging.value = true
      } else if (event.payload.type === 'leave') {
        isDragging.value = false
      } else if (event.payload.type === 'drop') {
        isDragging.value = false
        debugDropFlash.value = true
        setTrackedTimeout(
          () => (debugDropFlash.value = false),
          DEBUG_DROP_FLASH_DURATION_MS,
          activeTimeoutIds,
        )

        // If showing completion, close it and start new analysis
        if (store.showCompletion) {
          store.clearInstallResult()
        }

        const paths = event.payload.paths
        logDebug(`Dropped paths from Tauri: ${paths.join(', ')}`, 'drag-drop')

        if (paths && paths.length > 0) {
          try {
            await analyzeFiles(paths)
          } catch (error) {
            logError(`Failed to analyze dropped files: ${error}`, 'drag-drop')
            modal.showError(getErrorMessage(error))
          }
        }
      }
    })
    logDebug('Tauri drag-drop listener registered', 'drag-drop')
  } catch (error) {
    logError(`Failed to setup Tauri drag-drop listener: ${error}`, 'drag-drop')
  }

  // Listen for installation progress events
  try {
    unlistenProgress = await listen<InstallProgress>('install-progress', (event) => {
      const payload = event.payload
      if (payload.activeTasks) {
        // Debug log for parallel mode progress
        const activeInfo = payload.activeTasks
          .map((t) => `${t.taskIndex}(${t.taskName})=${t.percentage.toFixed(1)}%/${t.phase}`)
          .join(', ')
        const completedIds = payload.completedTaskIds?.join(', ') ?? ''
        logDebug(
          `[PROGRESS] Parallel mode: overall ${payload.percentage.toFixed(1)}%, active=[${activeInfo}], completed=${payload.completedTaskCount}/${payload.totalTasks}, completedIds=[${completedIds}], phase: ${payload.phase}`,
          'install',
        )
      } else {
        // Debug log for serial mode progress
        logDebug(
          `[PROGRESS] Serial mode: task ${payload.currentTaskIndex + 1}/${payload.totalTasks}, phase: ${payload.phase}, percentage: ${payload.percentage.toFixed(1)}%`,
          'install',
        )
      }
      progressStore.update(payload)
    })
    logDebug('Progress listener registered', 'install')
  } catch (error) {
    logError(`Failed to setup progress listener: ${error}`, 'install')
  }

  // Listen for source deletion skipped events
  try {
    unlistenDeletionSkipped = await listen<string>('source-deletion-skipped', (event) => {
      const path = event.payload
      toast.info(t('home.sourceDeletionSkipped', { path }))
    })
    logDebug('Source deletion skipped listener registered', 'install')
  } catch (error) {
    logError(`Failed to setup source deletion skipped listener: ${error}`, 'install')
  }

  // Note: Pending CLI args are now handled by the watcher above
  // No need to manually check here - the watcher will trigger automatically

  // Check if X-Plane is running on mount and periodically
  const checkXPlaneRunning = async () => {
    try {
      isXPlaneRunning.value = await invoke<boolean>('is_xplane_running')
    } catch (error) {
      logDebug(`Failed to check X-Plane running status: ${error}`, 'app')
    }
  }
  await checkXPlaneRunning()
  xplaneCheckInterval = window.setInterval(checkXPlaneRunning, 3000) // Check every 3 seconds

  if (store.xplanePath) {
    if (managementStore.aircraft.length === 0) managementStore.loadAircraft().catch(() => {})
    if (managementStore.plugins.length === 0) managementStore.loadPlugins().catch(() => {})
    if (sceneryStore.entries.length === 0) sceneryStore.loadData().catch(() => {})
  }
})

onBeforeUnmount(() => {
  window.removeEventListener('dragover', onWindowDragOver)
  window.removeEventListener('dragleave', onWindowDragLeave)
  window.removeEventListener('drop', onWindowDrop)
  window.removeEventListener('focus', onWindowFocus)
  window.removeEventListener('blur', onWindowBlur)

  // Cleanup all tracked timeouts to prevent memory leaks
  activeTimeoutIds.forEach((id) => clearTimeout(id))
  activeTimeoutIds.clear()

  // Cleanup Tauri listeners
  if (unlistenDragDrop) {
    unlistenDragDrop()
  }
  if (unlistenProgress) {
    unlistenProgress()
  }
  if (unlistenDeletionSkipped) {
    unlistenDeletionSkipped()
  }
  if (xplaneCheckInterval !== null) {
    clearInterval(xplaneCheckInterval)
    xplaneCheckInterval = null
  }
})

async function analyzeFiles(paths: string[], passwords?: Record<string, string>) {
  // Log incoming files
  logOperation('Files dropped for analysis', `${paths.length} file(s)`)
  logDebug(`Analyzing paths: ${paths.join(', ')}`, 'analysis')

  // Reset password retry counter for new analysis (not a retry with passwords)
  if (!passwords || Object.keys(passwords).length === 0) {
    passwordRetryCount.value = 0
  }

  if (!store.xplanePath) {
    logDebug('No X-Plane path set', 'analysis')
    // Log the abort reason - toast.warning will also log via the store
    logOperation('Task aborted', 'X-Plane path not set')
    toast.warning(t('home.pathNotSet'))
    return
  }

  store.isAnalyzing = true
  logDebug(`Starting analysis with X-Plane path: ${store.xplanePath}`, 'analysis')

  try {
    logDebug(`Paths to analyze: ${paths.join(', ')}`, 'analysis')

    const result = await invoke<AnalysisResult>('analyze_addons', {
      paths,
      xplanePath: store.xplanePath,
      passwords: passwords || null,
      verificationPreferences: store.verificationPreferences,
    })

    logDebug(
      `Analysis result: ${result.tasks.length} tasks, ${result.errors.length} errors`,
      'analysis',
    )

    // Check if any archives (including nested archives) require passwords
    const nestedRequiredPaths = result.nestedPasswordRequired
      ? Object.keys(result.nestedPasswordRequired)
      : []
    const allRequiredPaths = [...(result.passwordRequired || []), ...nestedRequiredPaths]

    if (allRequiredPaths.length > 0) {
      // Log password requirement
      logOperation('Password required', `${allRequiredPaths.length} file(s)`)
      logDebug(`Password required for: ${allRequiredPaths.join(', ')}`, 'analysis')
      // Store the original paths for re-analysis after password input
      pendingAnalysisPaths.value = paths
      passwordRequiredPaths.value = allRequiredPaths
      // Preserve already collected passwords
      if (passwords) {
        collectedPasswords.value = { ...passwords }
      }
      showPasswordModal.value = true
      store.isAnalyzing = false
      return
    }

    if (result.errors.length > 0) {
      logDebug(`Errors during analysis: ${result.errors.join('; ')}`, 'analysis')
      // Check if errors indicate wrong password
      const passwordErrors = result.errors.filter(
        (err) => err.includes('Wrong password') || err.toLowerCase().includes('wrong password'),
      )

      logDebug(
        `Password errors found: ${passwordErrors.length}, passwords param: ${passwords ? 'provided' : 'null'}`,
        'analysis',
      )

      if (passwordErrors.length > 0 && passwords && Object.keys(passwords).length > 0) {
        // Increment retry counter BEFORE checking limit to prevent race condition
        passwordRetryCount.value++

        logDebug(
          `Password retry count: ${passwordRetryCount.value}/${MAX_PASSWORD_RETRIES}`,
          'analysis',
        )

        // Check if we've exceeded retry limit (use >= to prevent off-by-one error)
        if (passwordRetryCount.value >= MAX_PASSWORD_RETRIES) {
          logOperation('Task aborted', 'Maximum password attempts exceeded')
          modal.showError(t('password.maxRetries') + '\n\n' + result.errors.join('\n'))
          resetPasswordState()
          store.isAnalyzing = false
          return
        }

        // Extract wrong password file paths from error messages
        const wrongPasswordPaths = extractWrongPasswordPaths(passwordErrors)
        logDebug(`Wrong password paths extracted: ${wrongPasswordPaths.join(', ')}`, 'analysis')
        if (wrongPasswordPaths.length > 0) {
          passwordRequiredPaths.value = wrongPasswordPaths
        }

        // Set error message and re-show password modal
        passwordErrorMessage.value = t('password.wrongPassword')
        showPasswordModal.value = true
        store.isAnalyzing = false
        return
      }

      if (result.tasks.length > 0) {
        // Non-blocking warning: keep valid tasks and continue install flow.
        const warningSummary = t('home.partialAnalysisWarning', { count: result.errors.length })
        logOperation(warningSummary, result.errors.join(' | '))
        toast.warning(`${warningSummary} ${t('home.partialAnalysisHint')}`)
      } else {
        // Blocking only when nothing usable was detected.
        modal.showError(result.errors.join('\n'))
        return
      }
    }

    if (result.tasks.length > 0) {
      // Filter tasks based on preferences
      const allowedTasks = result.tasks.filter((task) => {
        // LuaScript follows Plugin preference
        const effectiveType = task.type === AddonType.LuaScript ? AddonType.Plugin : task.type
        return store.installPreferences[effectiveType]
      })
      const ignoredCount = result.tasks.length - allowedTasks.length

      logDebug(
        `Filtered tasks: ${allowedTasks.length} allowed, ${ignoredCount} ignored`,
        'analysis',
      )

      if (ignoredCount > 0) {
        toast.info(t('home.ignoredTasks', { count: ignoredCount }))
      }

      if (allowedTasks.length > 0) {
        if (showConfirmation.value) {
          // Confirmation modal already open - append new tasks
          const addedCount = store.appendTasks(allowedTasks)
          if (addedCount > 0) {
            toast.success(t('home.tasksAppended', { count: addedCount }))
          } else {
            toast.info(t('home.duplicateTasksIgnored'))
          }
        } else {
          // Fresh analysis - replace task list
          store.setCurrentTasks(allowedTasks)
          showConfirmation.value = true
        }
        // Reset password state on successful analysis
        resetPasswordState()
        // Non-blocking log call
        logBasic('Analysis completed', 'analysis')
        logOperation('Analysis completed', `${allowedTasks.length} task(s) ready`)
        logDebug(`Task types: ${allowedTasks.map((t) => t.type).join(', ')}`, 'analysis')
      } else if (ignoredCount > 0) {
        toast.warning(t('home.allIgnored'))
      } else {
        toast.warning(t('home.noValidAddons'))
      }
    } else {
      logDebug('No valid addons detected in analysis', 'analysis')
      toast.warning(t('home.noValidAddons'))
    }
  } catch (error) {
    // Non-blocking log call (also prints to console.error internally)
    logError(`Analysis failed: ${error}`, 'analysis')
    modal.showError(t('home.failedToAnalyze') + ': ' + getErrorMessage(error))
  } finally {
    store.isAnalyzing = false
  }
}

// Handle password modal submit
async function handlePasswordSubmit(passwords: Record<string, string>) {
  // Rate limiting: check if attempts are too frequent
  const now = Date.now()
  const recentAttempts = passwordAttemptTimestamps.value.filter(
    (t) => now - t < PASSWORD_RATE_LIMIT_WINDOW_MS, // Within rate limit window
  )

  if (recentAttempts.length > 0) {
    const lastAttempt = Math.max(...recentAttempts)
    const timeSinceLastAttempt = now - lastAttempt

    if (timeSinceLastAttempt < MIN_PASSWORD_ATTEMPT_DELAY_MS) {
      toast.warning(t('password.tooFast') || 'Please wait before trying again')
      return
    }
  }

  // Record this attempt
  passwordAttemptTimestamps.value.push(now)
  // Keep only recent attempts (last 10 seconds)
  passwordAttemptTimestamps.value = passwordAttemptTimestamps.value.filter(
    (t) => now - t < PASSWORD_RATE_LIMIT_WINDOW_MS,
  )

  showPasswordModal.value = false
  passwordErrorMessage.value = '' // Clear error message
  logOperation('Password entered', `${Object.keys(passwords).length} file(s)`)
  // Merge new passwords with previously collected ones
  const allPasswords = { ...collectedPasswords.value, ...passwords }

  // Note: retry counter is incremented only when password is wrong (in analyzeFiles)
  // Re-analyze with passwords
  await analyzeFiles(pendingAnalysisPaths.value, allPasswords)
}

// Handle password modal cancel
async function handlePasswordCancel() {
  showPasswordModal.value = false
  logOperation('Task aborted', 'User canceled password input')

  // After cancel, continue analyzing files that don't require password
  const nonPasswordPaths = pendingAnalysisPaths.value.filter(
    (p) => !passwordRequiredPaths.value.includes(p),
  )

  resetPasswordState()

  // If there are files that don't need password, continue analyzing them
  if (nonPasswordPaths.length > 0) {
    await analyzeFiles(nonPasswordPaths)
  }
}

// Extract wrong password file paths from error messages
function extractWrongPasswordPaths(errors: string[]): string[] {
  const paths: string[] = []
  for (const err of errors) {
    // Match "Wrong password for archive: {path}" format
    const match = err.match(/Wrong password for archive:\s*(.+)$/i)
    if (match && match[1]) {
      paths.push(match[1].trim())
    }
  }
  // If unable to extract, return current passwordRequiredPaths
  return paths.length > 0 ? paths : passwordRequiredPaths.value
}

// Reset password state
function resetPasswordState() {
  pendingAnalysisPaths.value = []
  passwordRequiredPaths.value = []
  collectedPasswords.value = {}
  passwordRetryCount.value = 0
  passwordErrorMessage.value = ''
}

async function handleInstall() {
  showConfirmation.value = false

  // Filter only enabled tasks
  const enabledTasks = store.currentTasks.filter((task) => store.getTaskEnabled(task.id))

  if (enabledTasks.length === 0) {
    toast.warning(t('home.noTasksEnabled'))
    return
  }

  // Store the tasks being installed for the progress overlay
  store.setInstallingTasks(enabledTasks)

  store.isInstalling = true
  // Non-blocking log call
  logBasic('Installation started', 'installation')
  logOperation('Installation started', `${enabledTasks.length} task(s) ready`)
  logDebug(
    `Installing ${enabledTasks.length} tasks: ${enabledTasks.map((t) => t.displayName).join(', ')}`,
    'installation',
  )

  try {
    if (!lockStore.isInitialized) {
      await lockStore.initStore()
    }

    // Prepare enabled tasks with overwrite and backup settings
    const allTasksWithSettings = store.getTasksWithOverwrite()
    const tasksWithOverwrite = allTasksWithSettings.filter((task) => store.getTaskEnabled(task.id))

    const overwriteCount = tasksWithOverwrite.filter((t) => t.shouldOverwrite).length
    if (overwriteCount > 0) {
      logDebug(`${overwriteCount} tasks will overwrite existing files`, 'installation')
    }

    const result = await invoke<InstallResult>('install_addons', {
      tasks: tasksWithOverwrite,
      atomicInstallEnabled: store.atomicInstallEnabled,
      xplanePath: store.xplanePath,
      deleteSourceAfterInstall: store.deleteSourceAfterInstall,
      autoSortScenery: store.autoSortScenery,
      lockedSceneryFolderNames: lockStore.getLockedItems('scenery'),
      parallelEnabled: store.parallelInstallEnabled,
      maxParallel: store.maxParallelTasks,
    })

    // Log results
    if (result.failedTasks === 0) {
      logBasic('Installation completed successfully', 'installation')
    } else if (result.successfulTasks > 0) {
      logBasic('Installation completed with partial failures', 'installation')
    } else {
      logError('All installation tasks failed', 'installation')
    }
    logOperation(
      `${result.successfulTasks}/${result.totalTasks} tasks completed successfully (failed: ${result.failedTasks})`,
    )

    // Log failed tasks if any
    if (result.failedTasks > 0) {
      result.taskResults
        .filter((r) => !r.success)
        .forEach((r) => {
          logError(`${r.taskName}: ${r.errorMessage}`, 'installation')
        })
    }

    // Ensure progress bar shows 100% before transitioning
    progressStore.setPercentage(100)

    // Save installation result immediately (this will trigger the animation)
    store.setInstallResult(result)

    // Keep isInstalling true for a brief moment to show the animation
    // Then set it to false after animation starts
    setTrackedTimeout(
      () => {
        store.isInstalling = false
        progressStore.reset()
      },
      COMPLETION_ANIMATION_DELAY_MS,
      activeTimeoutIds,
    )
  } catch (error) {
    // Non-blocking log call (also prints to console.error internally)
    logError(`${t('log.installationFailed')}: ${error}`, 'installation')
    modal.showError(t('home.installationFailed') + ': ' + getErrorMessage(error))
    store.isInstalling = false
    progressStore.reset()
  }
}

// Handle skip current task
async function handleSkipTask() {
  const confirmed = await showConfirmDialog({
    title: t('taskControl.skipConfirmTitle'),
    message: t('taskControl.skipConfirmMessage'),
    warning: t('taskControl.skipWarningClean'),
    confirmText: t('taskControl.confirmSkip'),
    cancelText: t('common.cancel'),
    type: 'warning',
  })

  if (confirmed) {
    try {
      await invoke('skip_current_task')
      toast.info(t('taskControl.taskSkipped'))
    } catch (error) {
      logError(`Failed to skip task: ${error}`, 'installation')
      modal.showError(getErrorMessage(error))
    }
  }
}

// Handle cancel all tasks
async function handleCancelInstallation() {
  const confirmed = await showConfirmDialog({
    title: t('taskControl.cancelConfirmTitle'),
    message: t('taskControl.cancelConfirmMessage'),
    warning: t('taskControl.cancelWarningClean'),
    confirmText: t('taskControl.confirmCancel'),
    cancelText: t('common.cancel'),
    type: 'danger',
  })

  if (confirmed) {
    try {
      await invoke('cancel_installation')
      toast.info(t('taskControl.tasksCancelled'))
    } catch (error) {
      logError(`Failed to cancel installation: ${error}`, 'installation')
      modal.showError(getErrorMessage(error))
    }
  }
}

// Show confirm dialog helper
function showConfirmDialog(options: {
  title: string
  message: string
  warning?: string
  confirmText: string
  cancelText: string
  type: 'warning' | 'danger'
}): Promise<boolean> {
  return new Promise((resolve) => {
    modal.showConfirm({
      title: options.title,
      message: options.message,
      warning: options.warning,
      confirmText: options.confirmText,
      cancelText: options.cancelText,
      type: options.type,
      onConfirm: () => resolve(true),
      onCancel: () => resolve(false),
    })
  })
}

// Handle launch X-Plane button click
async function handleLaunchXPlane() {
  if (isLaunchingXPlane.value || isXPlaneRunning.value) return

  // Load fresh scenery data to check for pending changes
  await sceneryStore.loadData()

  if (sceneryStore.hasChanges) {
    showLaunchConfirmDialog.value = true
  } else {
    await launchXPlane()
  }
}

// Launch X-Plane directly with loading state and timeout
async function launchXPlane() {
  if (isLaunchingXPlane.value || isXPlaneRunning.value) return

  isLaunchingXPlane.value = true
  const LAUNCH_TIMEOUT = 15000 // 15 seconds
  const SUCCESS_DELAY = 5000 // 5 seconds after success

  try {
    // Parse launch args from store (split by whitespace, filter empty)
    const args = store.xplaneLaunchArgs ? store.xplaneLaunchArgs.split(/\s+/).filter(Boolean) : []

    // Race between launch and timeout
    const launchPromise = invoke('launch_xplane', {
      xplanePath: store.xplanePath,
      args: args.length > 0 ? args : null,
    })
    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('timeout')), LAUNCH_TIMEOUT),
    )

    await Promise.race([launchPromise, timeoutPromise])

    // Keep loading state for 5 more seconds after success
    await new Promise((resolve) => setTimeout(resolve, SUCCESS_DELAY))

    showLaunchConfirmDialog.value = false
  } catch (error) {
    logError(`Failed to launch X-Plane: ${error}`, 'app')
    if (error instanceof Error && error.message === 'timeout') {
      modal.showError(t('home.launchTimeout'))
    } else {
      const errorMsg = getErrorMessage(error)
      // Check if it's an elevation required error
      if (errorMsg.startsWith('ELEVATION_REQUIRED:')) {
        const exePath = errorMsg.substring('ELEVATION_REQUIRED:'.length)
        modal.showError(
          t('home.elevationRequired.title') +
            '\n\n' +
            t('home.elevationRequired.causes') +
            '\n' +
            t('home.elevationRequired.cause1') +
            '\n' +
            t('home.elevationRequired.cause2') +
            '\n' +
            t('home.elevationRequired.cause3') +
            '\n\n' +
            t('home.elevationRequired.solutions') +
            '\n' +
            t('home.elevationRequired.solution1') +
            '\n' +
            t('home.elevationRequired.solution2') +
            '\n\n' +
            t('home.elevationRequired.path') +
            ' ' +
            exePath,
          '',
          { hideReport: true },
        )
      } else {
        modal.showError(t('home.launchFailed') + ': ' + errorMsg)
      }
    }
  } finally {
    isLaunchingXPlane.value = false
  }
}

// Apply scenery changes then launch X-Plane
async function applyAndLaunch() {
  if (isLaunchingXPlane.value) return

  isLaunchingXPlane.value = true

  try {
    await sceneryStore.applyChanges()
    // Reset loading state before calling launchXPlane since it will set it again
    isLaunchingXPlane.value = false
    await launchXPlane()
  } catch (error) {
    logError(`Failed to apply changes and launch: ${error}`, 'app')
    modal.showError(getErrorMessage(error))
    isLaunchingXPlane.value = false
  }
}

// Cancel launch confirmation dialog
function cancelLaunchDialog() {
  showLaunchConfirmDialog.value = false
}

function handleCompletionConfirm() {
  setTrackedTimeout(
    () => {
      store.clearInstallResult()
      store.clearTasks()
    },
    COMPLETION_ANIMATION_DELAY_MS,
    activeTimeoutIds,
  )
}
</script>

<style scoped>
/* debug drop visual */
.debug-drop {
  border-color: #10b981 !important; /* emerald */
  box-shadow: 0 0 30px rgba(16, 185, 129, 0.15) !important;
}

/* Animations */
@keyframes fade-in {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

@keyframes bounce-in {
  0% {
    opacity: 0;
    transform: scale(0.3);
  }
  50% {
    opacity: 1;
    transform: scale(1.05);
  }
  70% {
    transform: scale(0.9);
  }
  100% {
    opacity: 1;
    transform: scale(1);
  }
}

.animate-fade-in {
  animation: fade-in 0.6s ease-out;
}

.animate-bounce-in {
  animation: bounce-in 0.8s ease-out;
}

/* Drop zone styles */
.drop-zone-card {
  position: relative;
  overflow: hidden;
}

.drop-zone-card::before {
  content: '';
  position: absolute;
  top: 0;
  left: -100%;
  width: 100%;
  height: 100%;
  background: linear-gradient(90deg, transparent, rgba(59, 130, 246, 0.1), transparent);
  transition: left 0.6s;
}

.drop-zone-card:hover::before {
  left: 100%;
}

.drop-zone-card.drag-over {
  border-color: #3b82f6;
  background: linear-gradient(135deg, rgba(59, 130, 246, 0.1), rgba(147, 51, 234, 0.1));
  transform: scale(1.02);
  box-shadow: 0 0 40px rgba(59, 130, 246, 0.3);
}

.icon-container {
  filter: drop-shadow(0 0 10px rgba(59, 130, 246, 0.3));
}

/* Progress cards */
.progress-card {
  backdrop-filter: blur(10px);
}

/* Smooth progress animations */
.progress-container {
  /* Add padding to prevent clipping of drop-shadow */
  padding: 10px;
  margin: -10px;
  overflow: visible;
}

.progress-circle {
  transition: stroke-dashoffset 50ms cubic-bezier(0.4, 0, 0.2, 1);
  will-change: stroke-dashoffset;
}

.progress-bar {
  transition: width 50ms cubic-bezier(0.4, 0, 0.2, 1);
  will-change: width;
}

.progress-bar-glow {
  transition: width 50ms cubic-bezier(0.4, 0, 0.2, 1);
  will-change: width;
  background: theme('colors.emerald.500');
  filter: blur(6px);
  opacity: 0.7;
}

.dark .progress-bar-glow {
  background: theme('colors.emerald.400');
  opacity: 0.8;
}

.progress-text {
  transition: opacity 100ms ease-out;
  will-change: opacity;
}

/* Optimized pulse animation - uses opacity instead of drop-shadow to avoid overflow clipping */
@keyframes progress-pulse {
  0%,
  100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.progress-circle {
  animation: progress-pulse 2s ease-in-out infinite;
}
</style>
