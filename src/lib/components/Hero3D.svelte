<script lang="ts">
    import { onMount } from "svelte";
    import * as THREE from "three";

    let canvas: HTMLCanvasElement;

    onMount(() => {
        const scene = new THREE.Scene();
        const camera = new THREE.PerspectiveCamera(
            50,
            window.innerWidth / window.innerHeight,
            0.1,
            1000,
        );
        const renderer = new THREE.WebGLRenderer({
            canvas,
            antialias: true,
            alpha: true,
            powerPreference: "high-performance",
        });

        renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
        renderer.setSize(window.innerWidth, window.innerHeight);
        camera.position.z = 15;

        // --- Modern "Glass Gallery" Aesthetic ---

        // 1. Aurora Background (Dynamic Gradient)
        const auroraGroup = new THREE.Group();
        const auroraGeom = new THREE.SphereGeometry(25, 32, 32);
        const auroraMat = new THREE.MeshBasicMaterial({
            color: 0x0a0a1a,
            side: THREE.BackSide,
        });
        const auroraSphere = new THREE.Mesh(auroraGeom, auroraMat);
        scene.add(auroraSphere);

        // 2. Floating Glass Panels (The "Memory Frames")
        const panels: THREE.Mesh[] = [];
        const panelCount = 12;
        const boxGeom = new THREE.BoxGeometry(4, 5, 0.1);

        for (let i = 0; i < panelCount; i++) {
            const material = new THREE.MeshPhysicalMaterial({
                color: 0xffffff,
                metalness: 0,
                roughness: 0.1,
                transmission: 0.9, // This creates the "Glass" look
                thickness: 0.5,
                transparent: true,
                opacity: 0.3,
                reflectivity: 0.5,
                clearcoat: 1,
                clearcoatRoughness: 0.1,
            });

            const panel = new THREE.Mesh(boxGeom, material);
            panel.position.set(
                (Math.random() - 0.5) * 20,
                (Math.random() - 0.5) * 15,
                (Math.random() - 0.5) * 10,
            );
            panel.rotation.set(
                Math.random() * 2,
                Math.random() * 2,
                Math.random() * 2,
            );
            scene.add(panel);
            panels.push(panel);
        }

        // --- Cinematic Lighting ---
        const ambientLight = new THREE.AmbientLight(0xffffff, 0.2);
        scene.add(ambientLight);

        const blueLight = new THREE.DirectionalLight(0x3b82f6, 1);
        blueLight.position.set(-10, 10, 5);
        scene.add(blueLight);

        const purpleLight = new THREE.PointLight(0x8b5cf6, 2, 50);
        purpleLight.position.set(10, -5, 10);
        scene.add(purpleLight);

        const accentLight = new THREE.PointLight(0xffffff, 1, 20);
        scene.add(accentLight);

        // --- Interaction ---
        let mouseX = 0;
        let mouseY = 0;
        const handleMouseMove = (e: MouseEvent) => {
            mouseX = (e.clientX / window.innerWidth - 0.5) * 4;
            mouseY = (e.clientY / window.innerHeight - 0.5) * 4;
        };
        window.addEventListener("mousemove", handleMouseMove);

        const clock = new THREE.Clock();

        function animate() {
            const time = clock.getElapsedTime();

            // Animate Panels
            panels.forEach((p, i) => {
                p.rotation.y += 0.002 * (i % 2 === 0 ? 1 : -1);
                p.rotation.x += 0.001 * (i % 3 === 0 ? 1 : -1);
                p.position.y += Math.sin(time + i) * 0.005;
            });

            // Interactive Camera Move
            camera.position.x += (mouseX - camera.position.x) * 0.05;
            camera.position.y += (-mouseY - camera.position.y) * 0.05;
            camera.lookAt(scene.position);

            // Light rotation for refractive glints
            accentLight.position.x = Math.sin(time) * 10;
            accentLight.position.y = Math.cos(time) * 10;

            renderer.render(scene, camera);
            requestAnimationFrame(animate);
        }

        function handleResize() {
            camera.aspect = window.innerWidth / window.innerHeight;
            camera.updateProjectionMatrix();
            renderer.setSize(window.innerWidth, window.innerHeight);
        }

        window.addEventListener("resize", handleResize);
        animate();

        return () => {
            window.removeEventListener("resize", handleResize);
            window.removeEventListener("mousemove", handleMouseMove);
            renderer.dispose();
            panels.forEach((p) => {
                p.geometry.dispose();
                (p.material as THREE.Material).dispose();
            });
            auroraGeom.dispose();
            auroraMat.dispose();
        };
    });
</script>

<canvas bind:this={canvas} class="three-canvas"></canvas>

<style>
    .three-canvas {
        position: fixed;
        top: 0;
        left: 0;
        width: 100vw;
        height: 100vh;
        z-index: -1;
        pointer-events: none;
        background: radial-gradient(circle at center, #0a0a1f 0%, #000000 100%);
    }
</style>
