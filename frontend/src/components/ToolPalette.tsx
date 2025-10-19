import React from "react";
import Icon from "./Icon";
import { theme, styleUtils, createStyles } from "../theme";

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

  const styles = createStyles({
    container: {
      position: "fixed",
      left: theme.spacing[6],
      top: "50%",
      transform: "translateY(-50%)",
      ...styleUtils.glassPanel("medium"),
      backdropFilter: theme.backdropBlur.lg,
      border: `1px solid ${theme.colors.border.medium}`,
      zIndex: theme.zIndex.dropdown,
      padding: theme.spacing[1],
    },
  });

  const getToolButtonStyles = (isActive: boolean, isConnected: boolean) => ({
    width: "48px",
    height: "48px",
    border: "none",
    background: isActive ? `rgba(255, 151, 0, 0.5)` : "transparent",
    borderRadius: theme.radius.lg,
    color: isActive
      ? theme.colors.primary.orange
      : isConnected
      ? theme.colors.primary.blue
      : theme.colors.gray[700],
    cursor: isConnected ? "pointer" : "not-allowed",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
    transition: theme.transitions.fast,
    position: "relative",
    opacity: isConnected ? 1 : 0.5,
    fontFamily: theme.fonts.primary,
    outline: "none",
  });

  const handleMouseEnter = (
    e: React.MouseEvent<HTMLButtonElement>,
    isActive: boolean,
  ) => {
    if (isConnected && !isActive) {
      e.currentTarget.style.background = "rgba(0, 104, 255, 0.4)";
      e.currentTarget.style.color = theme.colors.primary.blue;
    }
  };

  const handleMouseLeave = (
    e: React.MouseEvent<HTMLButtonElement>,
    isActive: boolean,
  ) => {
    if (isConnected && !isActive) {
      e.currentTarget.style.background = "transparent";
      e.currentTarget.style.color = theme.colors.primary.blue;
    }
  };

  return (
    <div style={styles.container}>
      {tools.map((tool, index) => {
        const isActive =
          selectedTool === tool.id ||
          (tool.id === "assets" && showAssetLibrary);

        return (
          <button
            key={tool.id}
            type="button"
            tabIndex={-1} 
            onMouseDown={(e) => e.preventDefault()}
            onFocus={(e) => e.currentTarget.blur()} 
            onClick={() => onToolSelect(tool.id)}
            disabled={!isConnected}
            style={{
              ...getToolButtonStyles(isActive, isConnected),
              marginBottom: index < tools.length - 1 ? theme.spacing[1] : "0",
            } as React.CSSProperties}
            onMouseEnter={(e) => handleMouseEnter(e, isActive)}
            onMouseLeave={(e) => handleMouseLeave(e, isActive)}
          >
            <Icon
              name={tool.id}
              size={44}
              color={
                isActive
                  ? theme.colors.primary.orange
                  : isConnected
                  ? theme.colors.primary.blue
                  : theme.colors.gray[700]
              }
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
                  background: theme.colors.primary.orange,
                  borderRadius: theme.radius.sm,
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
