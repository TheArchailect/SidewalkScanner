import React, { useState } from "react";

const ScannerApp = () => {
  const [selectedTool, setSelectedTool] = useState("polygon");

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

      {/* Top Bar */}
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
            width: "8px",
            height: "8px",
            background: "#00ff88",
            borderRadius: "50%",
          }}
        ></div>

        <div
          style={{
            fontSize: "13px",
            color: "#999",
            display: "flex",
            alignItems: "center",
            gap: "16px",
          }}
        >
          <span style={{ color: "#00ff88" }}>60 fps</span>
        </div>
      </div>

      {/* Vertical Tool Palette */}
      <div
        style={{
          position: "fixed",
          left: "20px",
          top: "50%",
          transform: "translateY(-50%)",
          background: "rgba(0, 0, 0, 0.3)",
          backdropFilter: "blur(20px)",
          borderRadius: "6px",
          border: "1px solid rgba(255, 255, 255, 0.08)",
          zIndex: 10,
          padding: "8px",
        }}
      >
        {[
          {
            id: "polygon",
            icon: (
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <polygon points="12,2 22,8.5 22,15.5 12,22 2,15.5 2,8.5 12,2" />
              </svg>
            ),
          },
          {
            id: "knife",
            icon: (
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <path d="m18 6 4 4-6.5 6.5a2.12 2.12 0 0 1-3 0L2 6l8.5 8.5a2.12 2.12 0 0 0 3 0L18 6z" />
              </svg>
            ),
          },
        ].map((tool, index) => (
          <button
            key={tool.id}
            onClick={() => setSelectedTool(tool.id)}
            style={{
              width: "36px",
              height: "36px",
              border: "none",
              background:
                selectedTool === tool.id
                  ? "rgba(255, 255, 255, 0.15)"
                  : "transparent",
              borderRadius: "4px",
              color: selectedTool === tool.id ? "#fff" : "#666",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "all 0.15s ease",
              marginBottom: index < 1 ? "4px" : "0",
              position: "relative",
            }}
            onMouseEnter={(e) => {
              if (selectedTool !== tool.id) {
                e.target.style.background = "rgba(255, 255, 255, 0.08)";
                e.target.style.color = "#aaa";
              }
            }}
            onMouseLeave={(e) => {
              if (selectedTool !== tool.id) {
                e.target.style.background = "transparent";
                e.target.style.color = "#666";
              }
            }}
          >
            {tool.icon}
            {selectedTool === tool.id && (
              <div
                style={{
                  position: "absolute",
                  left: "-2px",
                  top: "50%",
                  transform: "translateY(-50%)",
                  width: "2px",
                  height: "20px",
                  background: "#fff",
                  borderRadius: "1px",
                }}
              ></div>
            )}
          </button>
        ))}
      </div>
    </div>
  );
};

export default ScannerApp;
