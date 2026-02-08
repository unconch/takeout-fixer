<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";
  import { fade, fly } from "svelte/transition";

  interface ScanResult {
    media_file: string;
    metadata_file: string | null;
    status: string;
  }

  interface ProgressEvent {
    current: number;
    total: number;
    filename: string;
    status: string;
  }

  interface RepairReport {
    fixed_photos: number;
    fixed_videos: number;
    gps_restored: number;
    solo_copied: number;
    failed: number;
  }

  let sourcePath = $state("");
  let destPath = $state("");
  let files = $state<ScanResult[]>([]);
  let analyzing = $state(false);
  let repairing = $state(false);
  let repairDone = $state(false);
  let copySolos = $state(true);
  let errorMsg = $state("");
  let isDragging = $state(false);

  // Progress tracking
  let progress = $state<ProgressEvent | null>(null);
  let startTime = $state(0);
  let remainingTime = $state("");
  let finalReport = $state<RepairReport | null>(null);
  let displayedReport = $state<RepairReport>({
    fixed_photos: 0,
    fixed_videos: 0,
    gps_restored: 0,
    solo_copied: 0,
    failed: 0,
  });

  function animateCounters() {
    if (!finalReport) return;
    const duration = 1500;
    const start = Date.now();
    const target = { ...finalReport };

    const step = () => {
      const now = Date.now();
      const progress = Math.min((now - start) / duration, 1);
      const ease = 1 - Math.pow(1 - progress, 3);

      displayedReport.fixed_photos = Math.floor(target.fixed_photos * ease);
      displayedReport.fixed_videos = Math.floor(target.fixed_videos * ease);
      displayedReport.gps_restored = Math.floor(target.gps_restored * ease);
      displayedReport.solo_copied = Math.floor(target.solo_copied * ease);
      displayedReport.failed = Math.floor(target.failed * ease);

      if (progress < 1) requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
  }

  onMount(() => {
    // Progress listener
    const unlistenProgress = listen<ProgressEvent>(
      "repair-progress",
      (event) => {
        progress = event.payload;
        updateEstimate(event.payload);
      },
    );

    // Native Drag and Drop
    const unlistenDrop = listen<{ paths: string[] }>(
      "tauri://drag-drop",
      (event) => {
        isDragging = false;
        const paths = event.payload.paths;
        if (paths && paths.length > 0) {
          sourcePath = paths[0];
          runAnalysis();
        }
      },
    );

    const unlistenDragEnter = listen("tauri://drag-enter", () => {
      isDragging = true;
    });

    const unlistenDragLeave = listen("tauri://drag-leave", () => {
      isDragging = false;
    });

    return () => {
      unlistenProgress.then((f) => f());
      unlistenDrop.then((f) => f());
      unlistenDragEnter.then((f) => f());
      unlistenDragLeave.then((f) => f());
    };
  });

  function updateEstimate(p: ProgressEvent) {
    if (p.current === 1) startTime = Date.now();
    const elapsed = Date.now() - startTime;
    if (p.current > 5) {
      const msPerFile = elapsed / p.current;
      const remainingFiles = p.total - p.current;
      const remainingMs = msPerFile * remainingFiles;
      const mins = Math.floor(remainingMs / 60000);
      const secs = Math.ceil((remainingMs % 60000) / 1000);
      remainingTime = `${mins}m ${secs}s remaining`;
    }
  }

  async function selectSource() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Google Takeout Folder",
    });
    if (selected) {
      sourcePath = selected as string;
      repairDone = false;
      finalReport = null;
    }
  }

  async function selectDest() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: "Select Destination Folder",
    });
    if (selected) {
      destPath = selected as string;
      repairDone = false;
      finalReport = null;
    }
  }

  async function runAnalysis() {
    if (!sourcePath) return;
    analyzing = true;
    errorMsg = "";
    repairDone = false;
    finalReport = null;
    try {
      files = await invoke("analyze_folder", { path: sourcePath });
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      analyzing = false;
    }
  }

  async function runRepair() {
    if (totalFilesToProcess === 0 || !destPath) return;
    repairing = true;
    errorMsg = "";
    remainingTime = "Calculating...";
    try {
      finalReport = await invoke("repair_all", {
        files,
        destPath,
        copySolos,
      });
      repairDone = true;
      animateCounters();
    } catch (e: any) {
      errorMsg = e.toString();
    } finally {
      repairing = false;
    }
  }

  const readyCount = $derived(files.filter((f) => f.metadata_file).length);
  const totalCount = $derived(files.length);
  const totalFilesToProcess = $derived(copySolos ? totalCount : readyCount);
  const progressPercent = $derived(
    progress ? (progress.current / progress.total) * 100 : 0,
  );
</script>

<div class="app-shell">
  <div class="blob blob-1"></div>
  <div class="blob blob-2"></div>

  {#if isDragging}
    <div class="drag-overlay" transition:fade={{ duration: 200 }}>
      <div class="overlay-content">
        <div class="icon-drop">📂</div>
        <h2>Drop Takeout Folder</h2>
        <p>Release to start analyzing library</p>
      </div>
    </div>
  {/if}

  <main class="content">
    <header class="hero" in:fly={{ y: -20, duration: 600 }}>
      <div class="badge">Version 1.0 (PRO)</div>
      <h1>Truth for your <span>Memories</span></h1>
      <p>
        Restore original dates and GPS locations. Locally, safely, permanently.
      </p>
    </header>

    <div class="grid-container" in:fly={{ y: 20, duration: 600, delay: 200 }}>
      <section class="config-panel">
        <div class="glass-card">
          <div class="section-header">
            <div class="icon-circle">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
                ><path
                  d="M4 14.899A7 7 0 1 1 15.71 8h1.79a4.5 4.5 0 0 1 2.5 8.242"
                /><path d="M12 12v9" /><path d="m8 17 4 4 4-4" /></svg
              >
            </div>
            <h2>Setup Workspace</h2>
          </div>

          <div class="input-stack">
            <div class="input-group">
              <div class="label-row">
                <label for="source-input">Takeout Folder</label>
                {#if sourcePath}<span class="check">✓</span>{/if}
              </div>
              <div class="field-btn">
                <input
                  id="source-input"
                  readonly
                  placeholder="Pick your Source folder..."
                  value={sourcePath}
                />
                <button
                  onclick={selectSource}
                  class="btn-icon"
                  aria-label="Select Source Folder"
                  title="Select Folder"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path
                      d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"
                    /></svg
                  >
                </button>
              </div>
            </div>

            <div class="input-group">
              <div class="label-row">
                <label for="dest-input">Destination Folder</label>
                {#if destPath}<span class="check">✓</span>{/if}
              </div>
              <div class="field-btn">
                <input
                  id="dest-input"
                  readonly
                  placeholder="Pick your Fixed folder..."
                  value={destPath}
                />
                <button
                  onclick={selectDest}
                  class="btn-icon"
                  aria-label="Select Destination Folder"
                  title="Select Folder"
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    width="18"
                    height="18"
                    viewBox="0 0 24 24"
                    fill="none"
                    stroke="currentColor"
                    stroke-width="2"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    ><path
                      d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6a2 2 0 0 1-1.94 1.5H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.93a2 2 0 0 1 1.66.9l.82 1.2a2 2 0 0 0 1.66.9H18a2 2 0 0 1 2 2v2"
                    /></svg
                  >
                </button>
              </div>
            </div>

            <div class="checkbox-group">
              <input type="checkbox" id="copy-solos" bind:checked={copySolos} />
              <label for="copy-solos"
                >Also copy files without metadata (Solo files)</label
              >
            </div>
          </div>

          <div class="main-action">
            <button
              onclick={runAnalysis}
              disabled={!sourcePath || analyzing || repairing}
              class="btn-primary full"
            >
              {#if analyzing}
                <span class="spinner"></span> Analyzing...
              {:else}
                Scan Files
              {/if}
            </button>
          </div>
        </div>
      </section>

      <section
        class="display-panel"
        in:fly={{ x: 20, duration: 600, delay: 400 }}
      >
        {#if repairing}
          <div class="glass-card active-repair">
            <div class="progress-section">
              <div class="prog-label">
                <span>Repairing: {progress?.filename || "Initializing..."}</span
                >
                <span>{Math.round(progressPercent)}%</span>
              </div>
              <div class="progress-container">
                <div
                  class="progress-bar"
                  style="width: {progressPercent}%"
                ></div>
              </div>
              <div class="prog-meta">
                <span
                  >{progress?.current || 0} / {progress?.total || 0} files</span
                >
                <span class="timer pulse">{remainingTime}</span>
              </div>
            </div>
          </div>
        {:else if repairDone && finalReport}
          <div class="glass-card report-card">
            <div class="success-header">
              <div class="success-icon">✓</div>
              <h2>Repair Complete</h2>
              <p>Your photo library has been restored successfully.</p>
            </div>

            <div class="report-grid">
              <div class="report-item">
                <span class="l">Photos Fixed</span>
                <span class="v">{displayedReport.fixed_photos}</span>
              </div>
              <div class="report-item">
                <span class="l">Videos Fixed</span>
                <span class="v">{displayedReport.fixed_videos}</span>
              </div>
              <div class="report-item">
                <span class="l">GPS Restored</span>
                <span class="v">{displayedReport.gps_restored}</span>
              </div>
              <div class="report-item">
                <span class="l">Solo Files Copied</span>
                <span class="v">{displayedReport.solo_copied}</span>
              </div>
              <div class="report-item fail">
                <span class="l">Errors</span>
                <span class="v">{displayedReport.failed}</span>
              </div>
            </div>

            <button
              onclick={() => {
                repairDone = false;
                finalReport = null;
              }}
              class="btn-primary">Start New Session</button
            >
          </div>
        {:else if totalCount > 0}
          <div class="results-layout">
            <div class="stats-row">
              <div class="stat-bubble">
                <div class="val">{totalCount}</div>
                <div class="lab">Detected</div>
              </div>
              <div class="stat-bubble success">
                <div class="val">{readyCount}</div>
                <div class="lab">Fixable</div>
              </div>
              <div class="stat-bubble warning">
                <div class="val">{totalCount - readyCount}</div>
                <div class="lab">Solo</div>
              </div>
            </div>

            <div class="glass-card file-card">
              <div class="list-header">
                <h3>Library Preview</h3>
                <span class="count-tag">Showing {files.length} items</span>
              </div>
              <div class="scroll-area">
                {#each files as file}
                  <div class="file-item">
                    <div class="file-info">
                      <div
                        class="file-icon {file.metadata_file ? 'has-meta' : ''}"
                      >
                        <svg
                          xmlns="http://www.w3.org/2000/svg"
                          width="14"
                          height="14"
                          viewBox="0 0 24 24"
                          fill="none"
                          stroke="currentColor"
                          stroke-width="2"
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          ><rect
                            width="18"
                            height="18"
                            x="3"
                            y="3"
                            rx="2"
                            ry="2"
                          /><polyline
                            points="11 3 11 11 14 8 17 11 17 3"
                          /></svg
                        >
                      </div>
                      <span class="name"
                        >{file.media_file.split(/[\\/]/).pop()}</span
                      >
                    </div>
                    <span
                      class="status-pill {file.metadata_file
                        ? 'p-ready'
                        : 'p-missing'}"
                    >
                      {file.metadata_file ? "Linked" : "Plain"}
                    </span>
                  </div>
                {/each}
              </div>
            </div>

            <div class="execution-bar">
              <button
                onclick={runRepair}
                disabled={totalFilesToProcess === 0 || !destPath || repairing}
                class="btn-vibrant"
              >
                Merge Metadata into {totalFilesToProcess} Files
              </button>
            </div>
          </div>
        {:else}
          <div class="empty-state">
            <div class="empty-icon">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="64"
                height="64"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="1"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="floating"
                ><rect width="18" height="18" x="3" y="3" rx="2" /><path
                  d="M7 8h10"
                /><path d="M7 12h10" /><path d="M7 16h10" /></svg
              >
            </div>
            <h3>Waiting for Library</h3>
            <p>
              Select your Google Takeout folder on the left to begin analysis.
            </p>
          </div>
        {/if}
      </section>
    </div>

    {#if errorMsg}
      <div class="toast error">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="16"
          height="16"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
          ><circle cx="12" cy="12" r="10" /><line
            x1="12"
            y1="8"
            x2="12"
            y2="12"
          /><line x1="12" y1="16" x2="12.01" y2="16" /></svg
        >
        <span>{errorMsg}</span>
      </div>
    {/if}
  </main>
</div>

<style>
  :root {
    --bg-dark: #070709;
    --accent-blue: #3b82f6;
    --accent-purple: #8b5cf6;
    --accent-pink: #d946ef;
    --text-primary: #ffffff;
    --text-secondary: #94a3b8;
    --glass: rgba(255, 255, 255, 0.03);
    --glass-border: rgba(255, 255, 255, 0.08);
    --glass-hover: rgba(255, 255, 255, 0.05);
    --success-green: #10b981;
    --warning-orange: #f59e0b;
    --error-red: #ef4444;
  }

  :global(body) {
    background-color: var(--bg-dark);
    color: var(--text-primary);
    margin: 0;
    font-family: "Plus Jakarta Sans", sans-serif;
    -webkit-font-smoothing: antialiased;
    overflow: hidden;
  }

  .app-shell {
    position: relative;
    width: 100vw;
    height: 100vh;
    background: radial-gradient(circle at 50% 50%, #111115 0%, #070709 100%);
  }

  .blob {
    position: absolute;
    width: 40vw;
    height: 40vw;
    filter: blur(100px);
    opacity: 0.12;
    z-index: 0;
    border-radius: 50%;
    animation: blobify 20s infinite alternate;
  }

  .blob-1 {
    top: -10%;
    left: -10%;
    background: var(--accent-blue);
  }
  .blob-2 {
    bottom: -10%;
    right: -10%;
    background: var(--accent-purple);
    animation-delay: -5s;
  }

  @keyframes blobify {
    0% {
      transform: translate(0, 0) scale(1);
    }
    100% {
      transform: translate(10%, 10%) scale(1.2);
    }
  }

  .drag-overlay {
    position: fixed;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    background: rgba(59, 130, 246, 0.15);
    backdrop-filter: blur(10px);
    z-index: 999;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    border: 4px dashed var(--accent-blue);
    margin: 20px;
    border-radius: 3rem;
    box-sizing: border-box;
  }

  .overlay-content {
    text-align: center;
    background: rgba(0, 0, 0, 0.6);
    padding: 3rem 5rem;
    border-radius: 2rem;
    border: 1px solid var(--glass-border);
    box-shadow: 0 20px 50px rgba(0, 0, 0, 0.5);
  }

  .icon-drop {
    font-size: 5rem;
    margin-bottom: 1rem;
    display: block;
  }

  .content {
    position: relative;
    z-index: 1;
    max-width: 1200px;
    margin: 0 auto;
    padding: 2.5rem;
    height: calc(100vh - 5rem);
    display: flex;
    flex-direction: column;
  }

  .hero {
    text-align: center;
    margin-bottom: 3rem;
  }
  .badge {
    display: inline-block;
    padding: 0.4rem 0.8rem;
    background: rgba(59, 130, 246, 0.1);
    border: 1px solid rgba(59, 130, 246, 0.2);
    border-radius: 2rem;
    color: var(--accent-blue);
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 1rem;
  }
  h1 {
    font-family: "Outfit", sans-serif;
    font-size: 3.5rem;
    font-weight: 800;
    margin: 0;
  }
  h1 span {
    background: linear-gradient(
      135deg,
      var(--accent-blue),
      var(--accent-purple)
    );
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }
  .hero p {
    color: var(--text-secondary);
    font-size: 1.1rem;
    margin-top: 1rem;
  }

  .grid-container {
    display: grid;
    grid-template-columns: 400px 1fr;
    gap: 2rem;
    flex: 1;
    overflow: hidden;
  }

  .glass-card {
    background: var(--glass);
    backdrop-filter: blur(20px);
    border: 1px solid var(--glass-border);
    border-radius: 2rem;
    padding: 2.25rem;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.5);
    transition: all 0.4s cubic-bezier(0.2, 0.8, 0.2, 1);
  }

  .glass-card:hover {
    border-color: rgba(255, 255, 255, 0.2);
    background: rgba(255, 255, 255, 0.05);
  }

  .section-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 2rem;
  }
  .icon-circle {
    width: 44px;
    height: 44px;
    background: rgba(59, 130, 246, 0.1);
    border-radius: 14px;
    display: flex;
    align-items: center;
    justify-content: center;
    color: var(--accent-blue);
  }
  h2 {
    font-size: 1.25rem;
    margin: 0;
    font-weight: 700;
  }

  .input-stack {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .input-group label {
    display: block;
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-secondary);
    margin-bottom: 0.6rem;
  }
  .label-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .check {
    color: var(--success-green);
    font-weight: 800;
  }

  .field-btn {
    display: flex;
    background: rgba(0, 0, 0, 0.2);
    border: 1px solid var(--glass-border);
    border-radius: 1rem;
    overflow: hidden;
  }
  input[readonly] {
    flex: 1;
    background: transparent;
    border: none;
    padding: 1rem;
    color: var(--text-primary);
    font-size: 0.9rem;
    outline: none;
  }
  .btn-icon {
    background: var(--glass-hover);
    border: none;
    border-left: 1px solid var(--glass-border);
    padding: 0 1.2rem;
    cursor: pointer;
    color: var(--text-secondary);
  }
  .btn-icon:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.1);
  }

  .checkbox-group {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    margin-top: 0.5rem;
  }
  .checkbox-group input {
    width: 18px;
    height: 18px;
    cursor: pointer;
  }
  .checkbox-group label {
    font-size: 0.85rem;
    color: var(--text-secondary);
    cursor: pointer;
  }

  .main-action {
    margin-top: 2.5rem;
  }
  .btn-primary {
    background: var(--accent-blue);
    color: white;
    font-weight: 700;
    padding: 1.1rem;
    border: none;
    border-radius: 1.25rem;
    cursor: pointer;
    transition: all 0.2s;
  }
  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
    transform: translateY(-2px);
    box-shadow: 0 8px 20px rgba(37, 99, 235, 0.4);
  }
  .btn-primary.full {
    width: 100%;
  }

  /* Display Panel */
  .display-panel {
    flex: 1;
    overflow: hidden;
    display: flex;
    flex-direction: column;
  }
  .results-layout {
    height: 100%;
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }
  .stats-row {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.25rem;
  }
  .stat-bubble {
    background: var(--glass);
    border: 1px solid var(--glass-border);
    padding: 1.5rem;
    border-radius: 1.5rem;
    text-align: center;
  }
  .stat-bubble .val {
    font-family: "Outfit", sans-serif;
    font-size: 2rem;
    font-weight: 800;
  }
  .stat-bubble .lab {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    font-weight: 800;
    margin-top: 0.25rem;
  }
  .stat-bubble.success .val {
    color: var(--success-green);
  }
  .stat-bubble.warning .val {
    color: var(--warning-orange);
  }

  .file-card {
    flex: 1;
    padding: 1.75rem;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .scroll-area {
    flex: 1;
    overflow-y: auto;
    padding-right: 0.8rem;
    margin-top: 1rem;
  }

  .scroll-area::-webkit-scrollbar {
    width: 6px;
  }
  .scroll-area::-webkit-scrollbar-track {
    background: transparent;
  }
  .scroll-area::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
  }
  .scroll-area::-webkit-scrollbar-thumb:hover {
    background: var(--accent-blue);
  }
  .file-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.85rem;
    background: rgba(255, 255, 255, 0.02);
    border-radius: 1rem;
    margin-bottom: 0.6rem;
  }
  .file-info {
    display: flex;
    align-items: center;
    gap: 1rem;
    overflow: hidden;
  }
  .file-icon {
    width: 34px;
    height: 34px;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255, 255, 255, 0.03);
    border-radius: 10px;
    color: var(--text-secondary);
  }
  .file-icon.has-meta {
    color: var(--accent-blue);
    background: rgba(59, 130, 246, 0.1);
  }
  .name {
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status-pill {
    font-size: 0.7rem;
    font-weight: 800;
    padding: 0.25rem 0.75rem;
    border-radius: 1rem;
  }
  .p-ready {
    background: rgba(16, 185, 129, 0.1);
    color: var(--success-green);
  }
  .p-missing {
    background: rgba(245, 158, 11, 0.1);
    color: var(--warning-orange);
  }

  .execution-bar {
    margin-top: 0.5rem;
  }
  .btn-vibrant {
    width: 100%;
    padding: 1.4rem;
    background: linear-gradient(
      135deg,
      var(--accent-blue),
      var(--accent-purple)
    );
    border: none;
    border-radius: 1.5rem;
    color: white;
    font-weight: 800;
    font-size: 1.1rem;
    cursor: pointer;
    box-shadow: 0 10px 30px rgba(59, 130, 246, 0.3);
    transition: all 0.3s;
  }
  .btn-vibrant:hover:not(:disabled) {
    transform: scale(1.02);
    box-shadow: 0 15px 40px rgba(59, 130, 246, 0.4);
  }

  /* Active Repair UI */
  .active-repair {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    text-align: center;
  }
  .progress-section {
    width: 100%;
    max-width: 500px;
  }
  .prog-label {
    display: flex;
    justify-content: space-between;
    font-weight: 700;
    margin-bottom: 1rem;
    font-size: 1rem;
  }
  .progress-container {
    width: 100%;
    height: 12px;
    background: rgba(255, 255, 255, 0.05);
    border-radius: 10px;
    overflow: hidden;
    margin-bottom: 1rem;
  }
  .progress-bar {
    height: 100%;
    background: linear-gradient(
      90deg,
      var(--accent-blue),
      var(--accent-purple)
    );
    transition: width 0.3s ease-out;
  }
  .prog-meta {
    display: flex;
    justify-content: space-between;
    color: var(--text-secondary);
    font-size: 0.9rem;
    font-weight: 600;
  }
  .timer {
    color: var(--accent-blue);
  }

  /* Report UI */
  .report-card {
    flex: 1;
    overflow-y: auto;
    text-align: center;
  }
  .success-header {
    margin-bottom: 2.5rem;
  }
  .success-icon {
    width: 64px;
    height: 64px;
    background: var(--success-green);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    margin: 0 auto 1.5rem;
    font-size: 2rem;
    font-weight: 800;
  }
  .report-grid {
    display: grid;
    grid-template-columns: repeat(2, 1fr);
    gap: 1.25rem;
    margin-bottom: 3rem;
  }
  .report-item {
    background: rgba(255, 255, 255, 0.02);
    padding: 1.5rem;
    border-radius: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }
  .report-item .l {
    font-size: 0.8rem;
    font-weight: 800;
    color: var(--text-secondary);
    text-transform: uppercase;
  }
  .report-item .v {
    font-family: "Outfit", sans-serif;
    font-size: 2.5rem;
    font-weight: 800;
    color: var(--accent-blue);
  }
  .report-item.fail .v {
    color: var(--error-red);
  }

  .empty-state {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    text-align: center;
    opacity: 0.5;
  }
  .empty-icon {
    margin-bottom: 2rem;
  }
  .floating {
    animation: float 4s ease-in-out infinite;
  }
  @keyframes float {
    0%,
    100% {
      transform: translateY(0);
    }
    50% {
      transform: translateY(-20px);
    }
  }

  .pulse {
    animation: pulse 2s infinite;
  }
  @keyframes pulse {
    0%,
    100% {
      opacity: 1;
    }
    50% {
      opacity: 0.5;
    }
  }

  .spinner {
    display: inline-block;
    width: 18px;
    height: 18px;
    border: 3px solid rgba(255, 255, 255, 0.3);
    border-radius: 50%;
    border-top-color: white;
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
