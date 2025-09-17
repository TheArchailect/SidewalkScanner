"use client";

import { useState, useEffect, useRef } from "react";
import { useWebRpc } from "../hooks/useWebRpc";
import AssetLibrary from "../components/AssetLibrary";
import ToolPalette from "../components/ToolPalette";
import PolygonToolPanel from "../components/PolygonSelection";

const ScannerApp: React.FC = () => {
  const [selectedTool, setSelectedTool] = useState<string | null>(null);
  const [showAssetLibrary, setShowAssetLibrary] = useState<boolean>(false);
  const [showPolygonPanel, setShowPolygonPanel] = useState<boolean>(false);
  const [renderMode, setRenderMode] = useState<string>("RGB");

  // Create ref and pass to hook
  const canvasRef = useRef<HTMLIFrameElement | null>(null);

  // Use the RPC hook with the ref
  const {
    fps,
    isConnected,
    selectTool,
    setRenderMode: sendRenderMode,
    onNotification,
    clearTool,
  } = useWebRpc(canvasRef);

  useEffect(() => {
    console.log("[ScannerApp] canvasRef current:", canvasRef.current);
  }, [canvasRef.current]);

  // Listen for tool state changes from Bevy
  useEffect(() => {
    onNotification("tool_state_changed", (params?: Record<string, any>) => {
      console.log("Tool state changed from Bevy:", params);

      if (!params?.active || params?.tool === "none") {
        setSelectedTool(null);
        setShowAssetLibrary(false);
        setShowPolygonPanel(false);
        return;
      }

      // Reflect active tool in UI
      if (params?.tool === "polygon" && params?.active) {
        setSelectedTool("polygon");
        setShowPolygonPanel(true);
        setShowAssetLibrary(false);
      } else if (params?.tool === "assets" && params?.active) {
        setSelectedTool("assets");
        setShowAssetLibrary(true);
        setShowPolygonPanel(false);
      } else if (params?.tool === "measure" && params?.active) {
        setSelectedTool("measure");
        setShowAssetLibrary(false);
        setShowPolygonPanel(false);
      }
    });
  }, [onNotification]);

  const handleToolSelect = async (toolId: string): Promise<void> => {
    // Show asset library only when assets tool is selected
    if (toolId === "assets") {
      setShowAssetLibrary(true);
    } else {
      setShowAssetLibrary(false);
    }

    if (toolId === "polygon") {
      setShowPolygonPanel(true);
    } else {
      setShowPolygonPanel(false);
    }

    // Always select the clicked tool (each tool is either on or off)
    setSelectedTool(toolId);

    // Send tool selection to Bevy via RPC
    try {
      await selectTool(toolId);
      console.log(`Tool ${toolId} activated`);
    } catch (error) {
      console.error(`Failed to select tool ${toolId}:`, error);
    }
  };

  const handleRenderModeChange = async (mode: string): Promise<void> => {
    setRenderMode(mode);
    try {
      await sendRenderMode(mode);
      console.log(`Render mode changed to: ${mode}`);
    } catch (error) {
      console.error(`Failed to change render mode to ${mode}:`, error);
    }
  };

  // Drop any current selection focus to iframe.
  const refocusCanvas = () => {
    try {
      const sel = window.getSelection?.();
      sel?.removeAllRanges();
      (document.activeElement as HTMLElement | null)?.blur?.();
    } catch {}
    // Do it twice across frames to cover reflows/rerenders.
    const focusNow = () => canvasRef.current?.focus();
    requestAnimationFrame(() => {
      focusNow();
      setTimeout(focusNow, 0);
    });
  };

  // Global key handling for A/D and Escape
  useEffect(() => {
    const swallow = (e: KeyboardEvent) => {
      const k = e.key;

      if (k === "a" || k === "A" || k === "d" || k === "D") {
        e.preventDefault();
        e.stopPropagation();
        (e as any).stopImmediatePropagation?.();
        refocusCanvas();
        return true;
      }

      if (k === "Escape" || k === "Esc") {
        e.preventDefault();
        e.stopPropagation();
        (e as any).stopImmediatePropagation?.();

        refocusCanvas();

        clearTool()
          .catch(console.error)
          .finally(() => {
            setSelectedTool(null);
            setShowAssetLibrary(false);
            setShowPolygonPanel(false);
            refocusCanvas();
          });
        return true;
      }
      return false;
    };

    const onKeyDown = (e: KeyboardEvent) => {
      swallow(e);
    };
    const onKeyUp = (e: KeyboardEvent) => {
      if (swallow(e)) refocusCanvas();
    };

    document.addEventListener("keydown", onKeyDown, { capture: true });
    document.addEventListener("keyup", onKeyUp, { capture: true });
    return () => {
      document.removeEventListener("keydown", onKeyDown as any, { capture: true } as any);
      document.removeEventListener("keyup", onKeyUp as any, { capture: true } as any);
    };
  }, [clearTool]);

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100vw",
        height: "100vh",
        background: "#000",
        userSelect: "none",       
        WebkitUserSelect: "none" as any,
        caretColor: "transparent",  
      }}
    >
      {/* WASM Canvas - Full Screen */}
      <iframe
        ref={canvasRef}
        tabIndex={-1} 
        src="./renderer/SidewalkScanner.html"
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          width: "100%",
          height: "100%",
          border: "none",
          zIndex: 1,
        }}
        title="Point Cloud Canvas"
      />

      {/* Top Bar with RPC Status and Live FPS */}
      <div
        style={{
          position: "fixed",
          top: 0,
          left: 0,
          right: 0,
          height: "50px",
          background: "rgba(0, 0, 0, 0.3)",
          backdropFilter: "blur(20px)",
          borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
          zIndex: 10,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0 24px",
        }}
      >
        <div
          style={{
            fontSize: "13px",
            color: "#999",
            display: "flex",
            alignItems: "end",
            gap: "16px",
            userSelect: "none",
          }}
        >
          <span style={{ color: "#00ff88" }}>
            {fps > 0 ? `${fps.toFixed(1)} fps` : "--"}
          </span>
        </div>

        <div
          style={{
            display: "flex",
            alignItems: "center",
            gap: "8px",
            userSelect: "none",
          }}
        >
          <span
            style={{
              fontSize: "13px",
              color: "#999",
              marginRight: "8px",
            }}
          >
            Render Mode:
          </span>
          {["original", "modified", "RGB"].map((mode) => (
            <button
              key={mode}
              onClick={() => handleRenderModeChange(mode)}
              onMouseDown={(e) => e.preventDefault()}
              onFocus={(e) => e.currentTarget.blur()}
              style={{
                padding: "4px 12px",
                fontSize: "12px",
                border: "1px solid rgba(255, 255, 255, 0.2)",
                borderRadius: "4px",
                background:
                  renderMode === mode
                    ? "rgba(0, 255, 136, 0.2)"
                    : "rgba(255, 255, 255, 0.05)",
                color: renderMode === mode ? "#ffffff" : "#999",
                cursor: "pointer",
                transition: "all 0.2s ease",
              }}
              onMouseEnter={(e) => {
                if (renderMode !== mode) {
                  (e.target as HTMLButtonElement).style.background =
                    "rgba(255, 255, 255, 0.1)";
                  (e.target as HTMLButtonElement).style.color = "#ccc";
                }
              }}
              onMouseLeave={(e) => {
                if (renderMode !== mode) {
                  (e.target as HTMLButtonElement).style.background =
                    "rgba(255, 255, 255, 0.05)";
                  (e.target as HTMLButtonElement).style.color = "#999";
                }
              }}
            >
              {mode}
            </button>
          ))}
        </div>
      </div>

      {/* Tool Palette Component */}
      <ToolPalette
        selectedTool={selectedTool}
        showAssetLibrary={showAssetLibrary}
        onToolSelect={handleToolSelect}
        isConnected={isConnected}
      />

      {/* Asset Library Panel */}
      <AssetLibrary isVisible={showAssetLibrary} canvasRef={canvasRef} />
      <PolygonToolPanel isVisible={showPolygonPanel} canvasRef={canvasRef} />
    </div>
  );
};

export default ScannerApp;
