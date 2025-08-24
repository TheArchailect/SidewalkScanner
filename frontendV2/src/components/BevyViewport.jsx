import { useEffect, useRef } from "react";

// Guard to prevent double init in StrictMode
if (!window.__bevyBooted) window.__bevyBooted = false;

export default function BevyViewport() {
  const canvasRef = useRef(null);

  useEffect(() => {
    // Ensure Bevy's canvas exists
    if (canvasRef.current && canvasRef.current.id !== "bevy") {
      canvasRef.current.id = "bevy";
    }
    if (window.__bevyBooted) return;
    window.__bevyBooted = true;

    // Inline module code = bypasses Vite import analysis
    const script = document.createElement("script");
    script.type = "module";
    script.src = "/renderer/bootstrap.js";
    script.onerror = (e) => console.error("Failed to load Bevy bootstrap:", e);
    document.body.appendChild(script);

    return () => {
      document.body.removeChild(script);
    };
  }, []);

  return (
    <div style={{ width: "100%", height: "100%", position: "relative" }}>
      <canvas
        ref={canvasRef}
        id="bevy"
        style={{ display: "block", width: "100%", height: "100%" }}
      />
    </div>
  );
}
