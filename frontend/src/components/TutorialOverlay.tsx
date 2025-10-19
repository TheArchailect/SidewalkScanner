import type React from "react";
import { useState, useEffect, useRef } from "react";
import { theme, styleUtils, createStyles } from "../theme";

interface TutorialOverlayProps {
  isOpen: boolean;
  onClose: () => void;
}

const XIcon = () => (
  <svg
    style={{ height: "16px", width: "16px" }}
    fill="none"
    stroke="currentColor"
    viewBox="0 0 24 24"
  >
    <path
      strokeLinecap="round"
      strokeLinejoin="round"
      strokeWidth={2}
      d="M6 18L18 6M6 6l12 12"
    />
  </svg>
);

const CustomButton = ({
  children,
  onClick,
  disabled = false,
  variant = "default",
  size = "default",
  style = {},
}: {
  children: React.ReactNode;
  onClick?: () => void;
  disabled?: boolean;
  variant?: "default" | "outline" | "ghost";
  size?: "default" | "sm";
  style?: React.CSSProperties;
}) => {
  const getButtonStyles = () => {
    let baseStyle = {
      ...styleUtils.buttonBase(),
      opacity: disabled ? 0.5 : 1,
      cursor: disabled ? "not-allowed" : "pointer",
      pointerEvents: disabled ? ("none" as const) : ("auto" as const),
      ...style,
    };

    if (variant === "default") {
      baseStyle = {
        ...baseStyle,
        background: theme.colors.primary.blue,
        color: theme.colors.white,
      };
    }
    if (variant === "outline") {
      baseStyle = {
        ...baseStyle,
        border: `1px solid ${theme.colors.border.medium}`,
        background: theme.colors.background.input,
        color: theme.colors.white,
      };
    }

    if (size === "default") {
      baseStyle = {
        ...baseStyle,
        height: theme.spacing[10],
        padding: `${theme.spacing[2]} ${theme.spacing[5]}`,
        fontSize: theme.fontSizes.base,
      };
    }
    if (size === "sm") {
      baseStyle = {
        ...baseStyle,
        height: theme.spacing[8],
        padding: `${theme.spacing[2]} ${theme.spacing[4]}`,
        fontSize: theme.fontSizes.sm,
      };
    }

    return baseStyle;
  };

  const handleMouseEnter = (e: React.MouseEvent<HTMLButtonElement>) => {
    if (disabled) return;
    if (variant === "default") {
      e.currentTarget.style.background = theme.colors.primary.blueDark;
    } else if (variant === "outline") {
      e.currentTarget.style.background = theme.colors.background.hover;
    }
  };

  const handleMouseLeave = (e: React.MouseEvent<HTMLButtonElement>) => {
    if (disabled) return;
    if (variant === "default") {
      e.currentTarget.style.background = theme.colors.primary.blue;
    } else if (variant === "outline") {
      e.currentTarget.style.background = theme.colors.background.input;
    }
  };

  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={getButtonStyles()}
      onMouseEnter={handleMouseEnter}
      onMouseLeave={handleMouseLeave}
    >
      {children}
    </button>
  );
};

export function TutorialOverlay({ isOpen, onClose }: TutorialOverlayProps) {
  const [highlightedElement, setHighlightedElement] = useState<string | null>(
    null,
  );
  const highlightRef = useRef<HTMLDivElement>(null);

  const findAndHighlightElement = (selector: string) => {
    let element: Element | null = null;

    if (selector.includes("fps")) {
      element = document.querySelector('[style*="fps"]');
    } else if (selector.includes("Render Mode")) {
      element =
        document.querySelector('[style*="Render Mode"]')?.parentElement || null;
    } else if (selector.includes("left: 0")) {
      element = document.querySelector('[style*="left: \\"16px\\""]');
    }

    if (element && highlightRef.current) {
      const rect = element.getBoundingClientRect();
      const highlight = highlightRef.current;

      highlight.style.display = "block";
      highlight.style.top = `${rect.top - 4}px`;
      highlight.style.left = `${rect.left - 4}px`;
      highlight.style.width = `${rect.width + 8}px`;
      highlight.style.height = `${rect.height + 8}px`;
    }
  };

  const hideHighlight = () => {
    if (highlightRef.current) {
      highlightRef.current.style.display = "none";
    }
    setHighlightedElement(null);
  };

  const handleElementHover = (elementType: string, selector: string) => {
    setHighlightedElement(elementType);
    findAndHighlightElement(selector);
  };

  useEffect(() => {
    if (!isOpen) {
      hideHighlight();
    }
  }, [isOpen]);

  if (!isOpen) return null;

  const styles = createStyles({
    overlay: {
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: theme.colors.background.overlay,
      backdropFilter: theme.backdropBlur.base,
      display: "flex",
      justifyContent: "center",
      alignItems: "center",
      zIndex: theme.zIndex.overlay,
    },
    modal: {
      ...styleUtils.glassPanel("dark"),
      padding: theme.spacing[8],
      maxWidth: "800px",
      width: "90%",
      maxHeight: "80vh",
      overflowY: "auto",
      color: theme.colors.white,
      position: "relative",
    },
    header: {
      display: "flex",
      alignItems: "center",
      justifyContent: "space-between",
      marginBottom: theme.spacing[6],
      borderBottom: `1px solid ${theme.colors.border.default}`,
      paddingBottom: theme.spacing[4],
    },
    title: {
      ...styleUtils.text.title(),
      margin: 0,
    },
    closeButton: {
      background: "none",
      border: "none",
      color: theme.colors.gray[300],
      cursor: "pointer",
      padding: theme.spacing[1],
      borderRadius: theme.radius.base,
      transition: theme.transitions.base,
    },
    content: {
      display: "grid",
      gridTemplateColumns: "1fr 1fr",
      gap: theme.spacing[8],
    },
    section: {
      marginBottom: theme.spacing[6],
    },
    sectionTitle: {
      ...styleUtils.text.subtitle(),
      marginBottom: theme.spacing[3],
      borderBottom: `1px solid ${theme.colors.border.orange}`,
      paddingBottom: theme.spacing[2],
    },
    controlItem: {
      display: "flex",
      alignItems: "flex-start",
      marginBottom: theme.spacing[3],
      padding: theme.spacing[2],
      borderRadius: theme.radius.md,
      transition: theme.transitions.base,
      cursor: "pointer",
    },
    controlKey: {
      background: `rgba(255, 151, 0, 0.2)`,
      color: theme.colors.primary.orange,
      padding: `${theme.spacing[0]} ${theme.spacing[2]}`,
      borderRadius: theme.radius.base,
      fontSize: theme.fontSizes.xs,
      fontWeight: theme.fontWeights.semibold,
      minWidth: "fit-content",
      marginRight: theme.spacing[3],
      border: `1px solid ${theme.colors.border.orange}`,
      fontFamily: theme.fonts.mono,
    },
    controlDesc: {
      ...styleUtils.text.body(),
      lineHeight: "1.4",
    },
    toolItem: {
      padding: theme.spacing[3],
      borderRadius: theme.radius.md,
      marginBottom: theme.spacing[2],
      border: `1px solid ${theme.colors.border.default}`,
      transition: theme.transitions.base,
      cursor: "pointer",
    },
    toolName: {
      ...styleUtils.text.subtitle(),
      fontSize: theme.fontSizes.base,
      marginBottom: theme.spacing[1],
    },
    toolDesc: {
      ...styleUtils.text.caption(),
      lineHeight: "1.3",
    },
    highlight: {
      position: "fixed",
      pointerEvents: "none",
      border: `2px solid ${theme.colors.primary.orange}`,
      borderRadius: theme.radius.lg,
      animation: "pulse 2s infinite",
      zIndex: theme.zIndex.highlight,
      display: "none",
    },
    toolControls: {
      ...styleUtils.text.caption(),
      fontSize: theme.fontSizes.xs,
      color: theme.colors.gray[300],
      marginTop: theme.spacing[2],
      paddingTop: theme.spacing[2],
      borderTop: `1px solid ${theme.colors.border.default}`,
      lineHeight: "1.4",
    },
  });

  return (
    <>
      <div style={styles.overlay}>
        <div style={styles.modal}>
          {/* Header */}
          <div style={styles.header}>
            <h2 style={styles.title}>Scanner Controls & Tools</h2>
            <button
              onClick={onClose}
              style={styles.closeButton}
              onMouseEnter={(e) =>
                (e.currentTarget.style.color = theme.colors.gray[100])
              }
              onMouseLeave={(e) =>
                (e.currentTarget.style.color = theme.colors.gray[300])
              }
            >
              <XIcon />
            </button>
          </div>

          <div style={styles.content}>
            {/* Left Column - Controls */}
            <div>
              {/* Navigation Controls */}
              <div style={styles.section}>
                <h3 style={styles.sectionTitle}>Navigation</h3>

                <div style={styles.controlItem}>
                  <span style={styles.controlKey}>Mouse</span>
                  <span style={styles.controlDesc}>
                    Right-click and drag to move around
                  </span>
                </div>

                <div style={styles.controlItem}>
                  <span style={styles.controlKey}>WASD</span>
                  <span style={styles.controlDesc}>
                    Move forward, left, back, right
                  </span>
                </div>

                <div style={styles.controlItem}>
                  <span style={styles.controlKey}>Scroll</span>
                  <span style={styles.controlDesc}>Zoom in and out</span>
                </div>

                <div style={styles.controlItem}>
                  <span style={styles.controlKey}>Q / E</span>
                  <span style={styles.controlDesc}>
                    Rotate around focus point
                  </span>
                </div>

                <div style={styles.controlItem}>
                  <span style={styles.controlKey}>
                    Page Up / Page Down || R / F
                  </span>
                  <span style={styles.controlDesc}>
                    Pitch around focus point
                  </span>
                </div>

                <div style={styles.controlItem}>
                  <span style={styles.controlKey}>ESC</span>
                  <span style={styles.controlDesc}>
                    Clear current tool selection
                  </span>
                </div>
              </div>

              {/* Render Modes */}
              <div style={styles.section}>
                <h3 style={styles.sectionTitle}>View Modes</h3>

                <div
                  style={styles.controlItem}
                  onMouseEnter={() =>
                    handleElementHover("render-modes", "Render Mode")
                  }
                  onMouseLeave={hideHighlight}
                >
                  <span style={styles.controlKey}>Original</span>
                  <span style={styles.controlDesc}>
                    View unedited scan data
                  </span>
                </div>

                <div
                  style={styles.controlItem}
                  onMouseEnter={() =>
                    handleElementHover("render-modes", "Render Mode")
                  }
                  onMouseLeave={hideHighlight}
                >
                  <span style={styles.controlKey}>Modified</span>
                  <span style={styles.controlDesc}>
                    View your changes and edits
                  </span>
                </div>

                <div
                  style={styles.controlItem}
                  onMouseEnter={() =>
                    handleElementHover("render-modes", "Render Mode")
                  }
                  onMouseLeave={hideHighlight}
                >
                  <span style={styles.controlKey}>RGB</span>
                  <span style={styles.controlDesc}>
                    View real captured colors
                  </span>
                </div>
              </div>
            </div>

            {/* Right Column - Tools */}
            <div>
              <div style={styles.section}>
                <h3 style={styles.sectionTitle}>Tools</h3>

                <div
                  style={styles.toolItem}
                  onMouseEnter={() => handleElementHover("tools", "left: 0")}
                  onMouseLeave={hideHighlight}
                >
                  <div style={styles.toolName}>Polygon Tool</div>
                  <div style={styles.toolDesc}>
                    Edit, hide and classify polygon areas in the scan
                  </div>
                  <div style={styles.toolControls}>
                    <strong>Controls:</strong> Left click to place points → Left
                    Shift or Complete button to finish → Use polygon UI to
                    reclassify or hide with class masks
                  </div>
                </div>

                <div
                  style={styles.toolItem}
                  onMouseEnter={() => handleElementHover("tools", "left: 0")}
                  onMouseLeave={hideHighlight}
                >
                  <div style={styles.toolName}>Measure Tool</div>
                  <div style={styles.toolDesc}>
                    Take measurements within the 3D environment
                  </div>
                </div>

                <div
                  style={styles.toolItem}
                  onMouseEnter={() => handleElementHover("tools", "left: 0")}
                  onMouseLeave={hideHighlight}
                >
                  <div style={styles.toolName}>Asset Library</div>
                  <div style={styles.toolDesc}>
                    Add 3D objects and assets to your scene
                  </div>
                  <div style={styles.toolControls}>
                    <strong>Controls:</strong> Select asset/model → Left click
                    viewport to place → Right click asset + scroll wheel to
                    rotate → Right click elsewhere to stop rotating
                  </div>
                </div>
              </div>

              {/* Performance Info */}
              <div style={styles.section}>
                <h3 style={styles.sectionTitle}>Performance</h3>

                <div
                  style={styles.controlItem}
                  onMouseEnter={() => handleElementHover("fps", "fps")}
                  onMouseLeave={hideHighlight}
                >
                  <span style={styles.controlKey}>FPS</span>
                  <span style={styles.controlDesc}>
                    Shows real-time performance
                  </span>
                </div>
              </div>
            </div>
          </div>

          <div style={{ marginTop: theme.spacing[8], textAlign: "center" }}>
            <CustomButton onClick={onClose}>Got it!</CustomButton>
          </div>
        </div>
      </div>

      {/* Dynamic Highlight Element */}
      <div ref={highlightRef} style={styles.highlight}>
        <style>{`
          @keyframes pulse {
            0% { opacity: 1; }
            50% { opacity: 0.5; }
            100% { opacity: 1; }
          }
        `}</style>
      </div>
    </>
  );
}
