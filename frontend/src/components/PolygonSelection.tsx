import type React from "react";
import { useState, useEffect, useRef, type RefObject } from "react";
import { useWebRpc } from "../hooks/useWebRpc";
import { theme, styleUtils } from "../theme";

interface PolygonToolPanelProps {
  isVisible: boolean;
  canvasRef: RefObject<HTMLIFrameElement | null>;
}

interface ClassCategory {
  id: string;
  name: string;
  color: string;
  items: ClassItem[];
}

interface ClassItem {
  id: string;
  name: string;
  selected: boolean;
}

type Operation = "hide" | "reclassify";

const PolygonToolPanel: React.FC<PolygonToolPanelProps> = ({
  isVisible,
  canvasRef,
}) => {
  const [operation, setOperation] = useState<Operation>("hide");

  const [categories, setCategories] = useState<ClassCategory[]>([
    {
      id: "vehicles",
      name: "Vehicles",
      color: theme.colors.primary.blue,
      items: [
        { id: "car", name: "Cars", selected: false },
        { id: "truck", name: "Trucks", selected: false },
        { id: "bike", name: "Bikes", selected: false },
      ],
    },
    {
      id: "vegetation",
      name: "Vegetation",
      color: theme.colors.success,
      items: [
        { id: "tree", name: "Trees", selected: false },
        { id: "grass", name: "Grass", selected: false },
        { id: "bush", name: "Bushes", selected: false },
      ],
    },
    {
      id: "infrastructure",
      name: "Infrastructure",
      color: theme.colors.primary.orange,
      items: [
        { id: "building", name: "Buildings", selected: false },
        { id: "road", name: "Roads", selected: false },
        { id: "sidewalk", name: "Sidewalks", selected: false },
      ],
    },
    {
      id: "furniture",
      name: "Street Furniture",
      color: "#8b5cf6",
      items: [
        { id: "bench", name: "Benches", selected: false },
        { id: "sign", name: "Signs", selected: false },
        { id: "light", name: "Lights", selected: false },
      ],
    },
  ]);

  const [targetCategory, setTargetCategory] = useState<string>("");
  const [targetItem, setTargetItem] = useState<string>("");

  const returnFocusToCanvas = (): void => {
    setTimeout(() => {
      if (canvasRef.current) {
        canvasRef.current.focus();
      }
    }, 100);
  };

  const toggleSourceItem = (categoryId: string, itemId: string) => {
    setCategories((prev) =>
      prev.map((cat) =>
        cat.id === categoryId
          ? {
              ...cat,
              items: cat.items.map((item) =>
                item.id === itemId
                  ? { ...item, selected: !item.selected }
                  : item,
              ),
            }
          : cat,
      ),
    );
    returnFocusToCanvas();
  };

  const handleApply = () => {
    const selectedItems = categories.flatMap((cat) =>
      cat.items
        .filter((item) => item.selected)
        .map((item) => ({
          category: cat.name,
          item: item.name,
        })),
    );

    console.log(`[v0] ${operation} operation:`, {
      selectedItems,
      targetCategory: operation === "reclassify" ? targetCategory : null,
      targetItem: operation === "reclassify" ? targetItem : null,
    });
  };

  const getSelectedCount = () => {
    return categories.reduce(
      (total, cat) => total + cat.items.filter((item) => item.selected).length,
      0,
    );
  };

  const hasAnySelection = getSelectedCount() > 0;

  // Escape handling inside the panel
  const panelRef = useRef<HTMLDivElement | null>(null);
  const { clearTool } = useWebRpc(canvasRef);

  const handleCancel = () => {
    clearTool()
      .catch(console.error)
      .finally(() => {
        // front-end UI will close from ScannerApps tool_state_changed handler
        setTimeout(() => canvasRef.current?.focus(), 0);
      });
  };

  useEffect(() => {
    if (isVisible) panelRef.current?.focus();
  }, [isVisible]);

  const handleKeyDownCapture = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape" || e.key === "Esc") {
      e.preventDefault();
      e.stopPropagation();
      handleCancel();
    }
  };

  if (!isVisible) return null;

  return (
    <div
      ref={panelRef}
      tabIndex={-1}
      onKeyDownCapture={handleKeyDownCapture}
      style={{
        position: "fixed",
        right: theme.spacing[6],
        top: "70px",
        width: "360px",
        ...styleUtils.glassPanel("medium"),
        zIndex: theme.zIndex.modal,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: theme.spacing[5],
          borderBottom: `1px solid ${theme.colors.border.default}`,
        }}
      >
        <h3
          style={{
            margin: `0 0 ${theme.spacing[5]} 0`,
            ...styleUtils.text.subtitle(),
            textAlign: "center",
          }}
        >
          Polygon Tool
        </h3>

        <div
          style={{
            display: "flex",
            gap: theme.spacing[2],
            marginBottom: theme.spacing[4],
          }}
        >
          <button
            onClick={() => {
              setOperation("hide");
              returnFocusToCanvas();
            }}
            style={{
              ...styleUtils.buttonBase(),
              background:
                operation === "hide"
                  ? theme.colors.background.overlay
                  : theme.colors.background.card,
              border: `1px solid ${operation === "hide" ? theme.colors.border.orangeStrong : theme.colors.border.light}`,
              color:
                operation === "hide"
                  ? theme.colors.primary.orangeLight
                  : theme.colors.gray[500],
              padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
              borderRadius: theme.radius.md,
              fontSize: theme.fontSizes.sm,
              fontWeight: theme.fontWeights.medium,
              transition: theme.transitions.fast,
              flex: 1,
            }}
          >
            Hide Points
          </button>
          <button
            onClick={() => {
              setOperation("reclassify");
              returnFocusToCanvas();
            }}
            style={{
              ...styleUtils.buttonBase(),
              background:
                operation === "reclassify"
                  ? theme.colors.background.overlay
                  : theme.colors.background.card,
              border: `1px solid ${operation === "reclassify" ? theme.colors.border.orangeStrong : theme.colors.border.light}`,
              color:
                operation === "reclassify"
                  ? theme.colors.primary.orangeLight
                  : theme.colors.gray[500],
              padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
              borderRadius: theme.radius.md,
              fontSize: theme.fontSizes.sm,
              fontWeight: theme.fontWeights.medium,
              transition: theme.transitions.fast,
              flex: 1,
            }}
          >
            Reclassify Points
          </button>
        </div>

        <div
          style={{
            ...styleUtils.text.caption(),
            textAlign: "center",
            lineHeight: "1.4",
            background: theme.colors.background.card,
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            borderRadius: theme.radius.base,
            border: `1px solid ${theme.colors.border.light}`,
          }}
        >
          {operation === "hide"
            ? "Choose which types to hide, or leave empty to hide all points in the polygon"
            : "Choose which types to reclassify, or leave empty to reclassify all points in the polygon"}
        </div>
      </div>

      <div
        style={{
          padding: `${theme.spacing[4]} ${theme.spacing[5]}`,
          flex: 1,
          maxHeight: "400px",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: theme.spacing[4],
        }}
      >
        {/* Source Selection Card */}
        <div
          style={{
            background: theme.colors.background.card,
            border: `1px solid ${theme.colors.border.default}`,
            borderRadius: theme.radius.md,
            padding: theme.spacing[4],
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              marginBottom: theme.spacing[4],
            }}
          >
            <h4
              style={{
                margin: 0,
                fontSize: theme.fontSizes.base,
                fontWeight: theme.fontWeights.semibold,
                color: theme.colors.white,
              }}
            >
              {operation === "hide" ? "What to Hide" : "What to Reclassify"}
            </h4>
          </div>

          {categories.map((category) => (
            <div key={category.id} style={{ marginBottom: theme.spacing[3] }}>
              <div
                style={{
                  fontSize: theme.fontSizes.sm,
                  color: category.color,
                  marginBottom: theme.spacing[1],
                  fontWeight: theme.fontWeights.medium,
                  display: "flex",
                  alignItems: "center",
                  gap: theme.spacing[1],
                }}
              >
                <div
                  style={{
                    width: "6px",
                    height: "6px",
                    borderRadius: "50%",
                    background: category.color,
                  }}
                />
                {category.name}
              </div>

              <div
                style={{
                  display: "flex",
                  flexWrap: "wrap",
                  gap: theme.spacing[1],
                  paddingLeft: "10px",
                }}
              >
                {category.items.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => toggleSourceItem(category.id, item.id)}
                    style={{
                      ...styleUtils.toolItem(item.selected),
                      padding: `${theme.spacing[1]} ${theme.spacing[3]}`,
                      fontSize: theme.fontSizes.xs,
                      fontWeight: item.selected
                        ? theme.fontWeights.medium
                        : theme.fontWeights.normal,
                    }}
                  >
                    {item.name}
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>

        <div
          style={{
            minHeight: operation === "reclassify" ? "100px" : "0px",
            overflow: "hidden",
            transition: theme.transitions.slow,
          }}
        >
          {operation === "reclassify" && (
            <div
              style={{
                background: theme.colors.background.overlay,
                border: `1px solid ${theme.colors.border.orangeStrong}`,
                borderRadius: theme.radius.md,
                padding: theme.spacing[4],
              }}
            >
              <h4
                style={{
                  margin: `0 0 ${theme.spacing[3]} 0`,
                  fontSize: theme.fontSizes.base,
                  fontWeight: theme.fontWeights.semibold,
                  color: theme.colors.primary.orangeLight,
                  display: "flex",
                  alignItems: "center",
                  gap: theme.spacing[2],
                }}
              >
                <span>→</span>
                Reclassify To
              </h4>

              <div
                style={{
                  display: "flex",
                  flexDirection: "column",
                  gap: theme.spacing[2],
                }}
              >
                <select
                  value={targetCategory}
                  onChange={(e) => {
                    setTargetCategory(e.target.value);
                    setTargetItem("");
                    returnFocusToCanvas();
                  }}
                  style={{
                    ...styleUtils.inputField(),
                    padding: `${theme.spacing[3]} ${theme.spacing[3]}`,
                    fontSize: theme.fontSizes.sm,
                    cursor: "pointer",
                  }}
                >
                  <option
                    value=""
                    style={{
                      background: theme.colors.gray[800],
                      color: theme.colors.white,
                    }}
                  >
                    Choose category...
                  </option>
                  {categories.map((cat) => (
                    <option
                      key={cat.id}
                      value={cat.id}
                      style={{
                        background: theme.colors.gray[800],
                        color: theme.colors.white,
                      }}
                    >
                      {cat.name}
                    </option>
                  ))}
                </select>

                {targetCategory && (
                  <select
                    value={targetItem}
                    onChange={(e) => {
                      setTargetItem(e.target.value);
                      returnFocusToCanvas();
                    }}
                    style={{
                      ...styleUtils.inputField(),
                      padding: `${theme.spacing[3]} ${theme.spacing[3]}`,
                      fontSize: theme.fontSizes.sm,
                      cursor: "pointer",
                    }}
                  >
                    <option
                      value=""
                      style={{
                        background: theme.colors.gray[800],
                        color: theme.colors.white,
                      }}
                    >
                      Choose specific type...
                    </option>
                    {categories
                      .find((cat) => cat.id === targetCategory)
                      ?.items.map((item) => (
                        <option
                          key={item.id}
                          value={item.id}
                          style={{
                            background: theme.colors.gray[800],
                            color: theme.colors.white,
                          }}
                        >
                          {item.name}
                        </option>
                      ))}
                  </select>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      <div
        style={{
          padding: `${theme.spacing[4]} ${theme.spacing[5]}`,
          borderTop: `1px solid ${theme.colors.border.default}`,
          display: "flex",
          gap: theme.spacing[3],
        }}
      >
        <button
          onClick={handleCancel}
          style={{
            ...styleUtils.buttonGhost(),
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            fontSize: theme.fontSizes.sm,
            fontWeight: theme.fontWeights.semibold,
            flex: "1",
          }}
        >
          Cancel
        </button>
        <button
          onClick={handleApply}
          disabled={
            operation === "reclassify" && (!targetCategory || !targetItem)
          }
          style={{
            ...styleUtils.buttonBase(),
            background:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? theme.colors.background.card
                : operation === "hide"
                  ? theme.colors.background.overlay
                  : theme.colors.background.overlay,
            border: `1px solid ${
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? theme.colors.border.light
                : operation === "hide"
                  ? theme.colors.border.orangeStrong
                  : theme.colors.border.blueStrong
            }`,
            color:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? theme.colors.gray[600]
                : operation === "hide"
                  ? theme.colors.primary.orangeLight
                  : theme.colors.primary.blueLight,
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            fontSize: theme.fontSizes.sm,
            fontWeight: theme.fontWeights.semibold,
            cursor:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? "not-allowed"
                : "pointer",
            transition: theme.transitions.fast,
            flex: "2",
          }}
        >
          {operation === "hide"
            ? hasAnySelection
              ? "Hide Selected"
              : "Hide All"
            : hasAnySelection
              ? "Reclassify Selected"
              : "Reclassify All"}
        </button>
      </div>
    </div>
  );
};

export default PolygonToolPanel;
