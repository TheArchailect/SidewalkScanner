import React from "react";
import Icon from "./Icon";

interface Tool {
  id: string;
}

interface ToolPaletteProps {
  selectedTool: string | null;
  showAssetLibrary: boolean;
  onToolSelect: (toolId: string) => void;
  isConnected: boolean;
}

const ToolPalette: React.FC<ToolPaletteProps> = ({
  selectedTool,
  showAssetLibrary,
  onToolSelect,
  isConnected,
}) => {
  const tools: Tool[] = [
    { id: "polygon" },
    { id: "measure" },
    { id: "assets" },
  ];

  return (
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
      {tools.map((tool, index) => {
        // Tool is active if it's selected OR if it's the assets tool and library is open
        const isActive =
          selectedTool === tool.id ||
          (tool.id === "assets" && showAssetLibrary);

        return (
          <button
            key={tool.id}
            onClick={() => onToolSelect(tool.id)}
            disabled={!isConnected}
            style={{
              width: "36px",
              height: "36px",
              border: "none",
              background: isActive
                ? "rgba(255, 255, 255, 0.15)"
                : "transparent",
              borderRadius: "4px",
              color: isActive ? "#fff" : isConnected ? "#666" : "#333",
              cursor: isConnected ? "pointer" : "not-allowed",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "all 0.15s ease",
              marginBottom: index < tools.length - 1 ? "4px" : "0",
              position: "relative",
              opacity: isConnected ? 1 : 0.5,
            }}
            onMouseEnter={(e: React.MouseEvent<HTMLButtonElement>) => {
              if (isConnected && !isActive) {
                e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
                e.currentTarget.style.color = "#aaa";
              }
            }}
            onMouseLeave={(e: React.MouseEvent<HTMLButtonElement>) => {
              if (isConnected && !isActive) {
                e.currentTarget.style.background = "transparent";
                e.currentTarget.style.color = "#666";
              }
            }}
          >
            <Icon
              name={tool.id}
              size={32}
              color={isActive ? "#fff" : isConnected ? "#666" : "#333"}
            />
            {isActive && (
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
              />
            )}
          </button>
        );
      })}
    </div>
  );
};

export default ToolPalette;
