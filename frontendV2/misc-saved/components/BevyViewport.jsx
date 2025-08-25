import { useEffect, useRef } from "react";

// Vite can resolve the URL that hash generated the WASM compiled Bevy engine
const bevyWasmURL = new URL("/renderer/point-cloud-wasm.js", import.meta.url);

// Avoid double initialziation in React StrictMode
if (!window.__bevyBooted) window.__bevyBooted = false;

export default function BevyViewport() {
    const canvasRef = useRef<HTMLCanvasElement | null>(null);

    useEffect(() => {
        if (canvasRef.current && canvasRef.current.id !== "bevy") {
            canvasRef.current.id = "bevy";
        }
        (async () => {
            try {
                await import(bevyWasmURL.pathname);
            } catch (e) {
                console.error("Failed to load Bevy WASM module:", e);
            }
        })();
    }), [];

    return (
        <div style={{ width: "100%", height: "100%", position: "relative"}}>
            <canvas
                ref={canvasRef}
                id="bevy"
                style={{ display: "block", width: "100%", height: "100%"}}
            />
        </div>
    );
}