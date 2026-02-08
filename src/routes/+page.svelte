  import Hero3D from "$lib/components/Hero3D.svelte";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";

  let visible = $state(false);
  let isTauri = $state(false);

  onMount(() => {
    // Detect if we are inside Tauri
    if (window && (window as any).__TAURI_INTERNALS__) {
      isTauri = true;
      goto("/fix");
    } else {
      setTimeout(() => (visible = true), 100);
    }
  });
</script>

{#if !isTauri}
<div class="landing-page" class:visible>
  <Hero3D />

  <nav class="glass-nav">
    <div class="logo">
      <div class="logo-box"></div>
      <span>Metadata Takeout Fixer</span>
    </div>
    <div class="nav-links">
      <a href="#features">Features</a>
      <a href="https://github.com/unconch/takeout-fixer" target="_blank"
        >GitHub</a
      >
      <a href="/fix" class="btn-sm">Launch App</a>
    </div>
  </nav>

  <main class="hero-content">
    <div class="badge">Privacy First • Local Only</div>
    <h1>Fix your <span>Google Photos</span></h1>
    <p class="subtitle">
      Google gives you your photos back broken. We fix them — properly and
      permanently. Merge metadata back into your files locally and safely.
    </p>

    <div class="cta-group">
      <a href="/fix" class="btn-primary-lg">
        Restore My Library
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
          ><path d="M5 12h14" /><path d="m12 5 7 7-7 7" /></svg
        >
      </a>
      <a href="#how-it-works" class="btn-secondary-lg">How it works</a>
    </div>

    <div class="hero-stats">
      <div class="stat">
        <span class="v">100k+</span>
        <span class="l">Files Handled</span>
      </div>
      <div class="divider"></div>
      <div class="stat">
        <span class="v">0ns</span>
        <span class="l">Cloud Latency</span>
      </div>
      <div class="divider"></div>
      <div class="stat">
        <span class="v">100%</span>
        <span class="l">Private</span>
      </div>
    </div>
  </main>

  <section id="features" class="features-section">
    <div class="section-title">
      <h2>Engineered for <span>Trust</span></h2>
    </div>
    <div class="feature-grid">
      <div class="feature-card glass">
        <div class="f-icon">🔒</div>
        <h3>Local-First</h3>
        <p>
          Your memories never leave your device. All processing happens on your
          CPU, ensuring 100% privacy.
        </p>
      </div>
      <div class="feature-card glass">
        <div class="f-icon">⚡</div>
        <h3>High Performance</h3>
        <p>
          Built with Rust to handle massive libraries of 100,000+ files with
          ease and safety.
        </p>
      </div>
      <div class="feature-card glass">
        <div class="f-icon">📸</div>
        <h3>EXIF Injection</h3>
        <p>
          Permanently writes dates and locations directly into your file headers
          for future-proof compatibility.
        </p>
      </div>
    </div>
  </section>
</div>

<style>
  :root {
    --accent-blue: #3b82f6;
    --accent-purple: #8b5cf6;
    --text-primary: #ffffff;
    --text-secondary: #94a3b8;
    --glass: rgba(255, 255, 255, 0.03);
    --glass-border: rgba(255, 255, 255, 0.08);
  }

  :global(body) {
    background-color: #000;
    color: var(--text-primary);
    margin: 0;
    font-family: "Outfit", sans-serif;
    overflow-x: hidden;
  }

  .landing-page {
    opacity: 0;
    transition: opacity 1s ease;
  }

  .landing-page.visible {
    opacity: 1;
  }

  .glass-nav {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 1.5rem 4rem;
    position: fixed;
    top: 0;
    width: calc(100% - 8rem);
    z-index: 100;
    background: rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(10px);
    border-bottom: 1px solid var(--glass-border);
  }

  .logo {
    display: flex;
    align-items: center;
    gap: 0.8rem;
    font-weight: 800;
    font-size: 1.2rem;
    letter-spacing: -0.02em;
  }

  .logo-box {
    width: 24px;
    height: 24px;
    background: linear-gradient(
      135deg,
      var(--accent-blue),
      var(--accent-purple)
    );
    border-radius: 6px;
    transform: rotate(45deg);
  }

  .nav-links {
    display: flex;
    align-items: center;
    gap: 2.5rem;
  }

  .nav-links a {
    color: var(--text-secondary);
    text-decoration: none;
    font-size: 0.9rem;
    font-weight: 500;
    transition: color 0.2s;
  }

  .nav-links a:hover {
    color: var(--text-primary);
  }

  .btn-sm {
    background: var(--text-primary);
    color: #000 !important;
    padding: 0.5rem 1.2rem;
    border-radius: 2rem;
    font-weight: 700 !important;
  }

  .hero-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    text-align: center;
    padding: 0 2rem;
  }

  .badge {
    background: rgba(59, 130, 246, 0.1);
    color: var(--accent-blue);
    padding: 0.5rem 1rem;
    border-radius: 2rem;
    font-weight: 700;
    font-size: 0.75rem;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 2rem;
    border: 1px solid rgba(59, 130, 246, 0.2);
  }

  h1 {
    font-size: 5rem;
    font-weight: 900;
    margin: 0;
    line-height: 1;
    letter-spacing: -0.04em;
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

  .subtitle {
    max-width: 600px;
    font-size: 1.25rem;
    color: var(--text-secondary);
    margin: 2rem 0 3rem;
    line-height: 1.6;
  }

  .cta-group {
    display: flex;
    gap: 1.5rem;
    align-items: center;
  }

  .btn-primary-lg {
    background: var(--text-primary);
    color: #000;
    padding: 1.2rem 2.5rem;
    border-radius: 4rem;
    font-size: 1.1rem;
    font-weight: 800;
    text-decoration: none;
    display: flex;
    align-items: center;
    gap: 0.8rem;
    transition:
      transform 0.2s,
      box-shadow 0.2s;
  }

  .btn-primary-lg:hover {
    transform: translateY(-4px);
    box-shadow: 0 10px 30px rgba(255, 255, 255, 0.2);
  }

  .btn-secondary-lg {
    color: var(--text-primary);
    padding: 1.2rem 2.5rem;
    border-radius: 4rem;
    font-size: 1.1rem;
    font-weight: 700;
    text-decoration: none;
    border: 1px solid var(--glass-border);
    transition: background 0.2s;
  }

  .btn-secondary-lg:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .hero-stats {
    display: flex;
    gap: 4rem;
    margin-top: 6rem;
    align-items: center;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 0.2rem;
  }

  .stat .v {
    font-size: 2rem;
    font-weight: 900;
  }
  .stat .l {
    font-size: 0.8rem;
    color: var(--text-secondary);
    text-transform: uppercase;
    font-weight: 700;
    letter-spacing: 0.1em;
  }

  .divider {
    width: 1px;
    height: 40px;
    background: var(--glass-border);
  }

  .features-section {
    padding: 8rem 4rem;
    max-width: 1200px;
    margin: 0 auto;
  }

  .section-title {
    text-align: center;
    margin-bottom: 4rem;
  }
  .section-title h2 {
    font-size: 3rem;
    font-weight: 800;
  }
  .section-title h2 span {
    background: linear-gradient(
      135deg,
      var(--accent-blue),
      var(--accent-purple)
    );
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .feature-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 2rem;
  }

  .feature-card {
    padding: 3rem;
    border-radius: 2rem;
    text-align: left;
    background: var(--glass);
    border: 1px solid var(--glass-border);
    transition: transform 0.3s ease;
  }

  .feature-card:hover {
    transform: translateY(-10px);
    border-color: rgba(59, 130, 246, 0.3);
  }

  .f-icon {
    font-size: 2.5rem;
    margin-bottom: 1.5rem;
    display: block;
  }
  h3 {
    font-size: 1.5rem;
    margin: 0 0 1rem;
    font-weight: 800;
  }
  .feature-card p {
    color: var(--text-secondary);
    line-height: 1.6;
    margin: 0;
  }

  @media (max-width: 768px) {
    .glass-nav {
      padding: 1rem 2rem;
      width: calc(100% - 4rem);
    }
    .nav-links {
      display: none;
    }
    h1 {
      font-size: 3rem;
    }
    .cta-group {
      flex-direction: column;
    }
    .feature-grid {
      grid-template-columns: 1fr;
    }
    .hero-stats {
      gap: 2rem;
      flex-wrap: wrap;
      justify-content: center;
    }
  }
</style>
{/if}
