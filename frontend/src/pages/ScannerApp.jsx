import { useState, useEffect } from "react";
import { useWebRpc } from "../hooks/useWebRpc";
import AssetLibrary from "../components/AssetLibrary";
import ToolPalette from "../components/ToolPalette";
const ScannerApp = () => {
  const [selectedTool, setSelectedTool] = useState("polygon");
  const [showAssetLibrary, setShowAssetLibrary] = useState(false);

  // Use the RPC hook
  const { fps, isConnected, selectTool, onNotification, canvasRef } =
    useWebRpc();

  // Listen for tool state changes from Bevy
  useEffect(() => {
    onNotification("tool_state_changed", (params) => {
      console.log("Tool state changed from Bevy:", params);
      if (params.tool === "polygon" && params.active) {
        setSelectedTool("polygon");
      }
    });
  }, [onNotification]);

  const handleToolSelect = async (toolId) => {
    if (toolId === "assets") {
      setShowAssetLibrary(true);
    } else {
      setShowAssetLibrary(false);
    }

    if (toolId == selectTool) {
      setSelectedTool(null);
    } else {
      setSelectedTool(toolId);
    }

    // Send tool selection to Bevy via RPC
    try {
      await selectTool(toolId);
      console.log(`Tool ${toolId} activated`);
    } catch (error) {
      console.error(`Failed to select tool ${toolId}:`, error);
    }
  };

  return (
    <div
      style={{
        position: "fixed",
        top: 0,
        left: 0,
        width: "100vw",
        height: "100vh",
        background: "#000",
      }}
    >
      {/* WASM Canvas - Full Screen */}
      <iframe
        ref={canvasRef}
        src="/renderer/SidewalkScanner.html"
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
          }}
        >
          <span style={{ color: "#00ff88" }}>
            {fps > 0 ? `${fps.toFixed(1)} fps` : "--"}
          </span>
        </div>
      </div>

      {/* Tool Palette Component */}
      <ToolPalette
        selectedTool={selectedTool}
        onToolSelect={handleToolSelect}
        isConnected={isConnected}
      />

      {/* Asset Library Panel */}
      <AssetLibrary isVisible={showAssetLibrary} canvasRef={canvasRef} />
    </div>
  );
};

export default ScannerApp;
