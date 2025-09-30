import type React from "react";
import { useState, useEffect, useRef, type RefObject } from "react";
import { useWebRpc } from "../hooks/useWebRpc";
import { theme, styleUtils } from "../theme";

interface PolygonToolPanelProps {
  isVisible: boolean;
  canvasRef: RefObject<HTMLIFrameElement | null>;
}

interface ClassItem {
  id: string;     // maps to asset id/name from availableAssets
  name: string;   // pretty label for the UI
  selected: boolean;
}

interface ClassCategory {
  id: string;         // maps to asset.category (or "uncategorized")
  name: string;       // pretty label for the UI
  color: string;      // UI color per category
  items: ClassItem[]; // assets belonging to this category
}

type Operation = "hide" | "reclassify";

/*const PolygonToolPanel: React.FC<PolygonToolPanelProps> = ({
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
  ]);*/

const PolygonToolPanel: React.FC<PolygonToolPanelProps> = ({
  isVisible,
  canvasRef,
}) => {
  const {
    // assets + RPC helpers (same baseline as AssetLibrary)
    availableAssets,
    getAvailableAssets,
    selectTool,
    clearTool,
    hidePointsInPolygon,
    reclassifyPointsInPolygon,
  } = useWebRpc(canvasRef);

  const [operation, setOperation] = useState<Operation>("hide");
  const [categories, setCategories] = useState<ClassCategory[]>([]);
  const [targetCategory, setTargetCategory] = useState<string>("");
  const [targetItem, setTargetItem] = useState<string>("");

  // -------- helpers --------
  const returnFocusToCanvas = (): void => {
    setTimeout(() => canvasRef.current?.focus(), 100);
  };

  // Convert snake_id into title labels - regex expression
  const prettify = (id: string) =>
    id.replace(/[_-]+/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());

  const colorForCategory = (catId: string) => {
    // Keep the same visual language as the old hard-coded set
    if (catId === "vehicles") return theme.colors.primary.blue;
    if (catId === "vegetation") return theme.colors.success;
    if (catId === "infrastructure") return theme.colors.primary.orange;
    if (catId === "furniture") return "#8b5cf6";
    if (catId === "uncategorized") return theme.colors.gray[400];
    return theme.colors.primary.blue;
  };

  const buildPolygonCategories = (): ClassCategory[] => {
    const groups = new Map<string, ClassItem[]>();

    for (const a of availableAssets || []) {
      const cat = a.category || "uncategorized";
      const id = (a.id || a.name || "").toString();
      if (!id) continue;

      const item: ClassItem = {
        id,
        name: prettify(a.name || id),
        selected: false,
      };

      if (!groups.has(cat)) groups.set(cat, []);
      groups.get(cat)!.push(item);
    }

    const out: ClassCategory[] = [];
    for (const [catId, items] of groups) {
      out.push({
        id: catId,
        name: prettify(catId),
        color: colorForCategory(catId),
        items: items.sort((a, b) => a.name.localeCompare(b.name)),
      });
    }

    // stable ordering by category name
    return out.sort((a, b) => a.name.localeCompare(b.name));
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

  const getSelectedPairs = () =>
    categories.flatMap((cat) =>
      cat.items
        .filter((i) => i.selected)
        .map((i) => ({ category_id: cat.id, item_id: i.id })),
    );

  const getSelectedCount = () =>
    categories.reduce(
      (sum, cat) => sum + cat.items.filter((i) => i.selected).length,
      0,
    );

  const hasAnySelection = getSelectedCount() > 0;

  const handleApply = async () => {
    const selectedPairs = getSelectedPairs();

    if (operation === "hide") {
      // Hide selected (or all if none selected) within the active polygon
      // Backend will interpret empty source_items as "all"
      await hidePointsInPolygon(selectedPairs);
      console.log("[v0] Hide function triggered successfully", selectedPairs)
    } else {
      // Reclassify selected (or all) to the chosen target
      if (!targetCategory || !targetItem) return;
      await reclassifyPointsInPolygon(selectedPairs, targetCategory, targetItem);
    }

    returnFocusToCanvas();
  };



  // When panel opens: activate tool + ensure assets are loaded
  useEffect(() => {
    if (!isVisible) return;
    selectTool("polygon").catch(console.error);
    getAvailableAssets().catch(console.error);
  }, [isVisible]);

  // Rebuild categories whenever availableAssets changes
  useEffect(() => {
    setCategories(buildPolygonCategories());
    // Reset target selection to avoid stale ids
    setTargetCategory("");
    setTargetItem("");
  }, [availableAssets]);

  // Escape handling inside the panel
  const panelRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (isVisible) panelRef.current?.focus();
  }, [isVisible]);

  const handleCancel = () => {
    clearTool()
      .catch(console.error)
      .finally(() => setTimeout(() => canvasRef.current?.focus(), 0));
  };

  const handleKeyDownCapture = (e: React.KeyboardEvent<HTMLDivElement>) => {
    if (e.key === "Escape" || e.key === "Esc") {
      e.preventDefault();
      e.stopPropagation();
      handleCancel();
    }
  };

  if (!isVisible) return null;

  // -------- render --------
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
              border: `1px solid ${
                operation === "hide"
                  ? theme.colors.border.orangeStrong
                  : theme.colors.border.light
              }`,
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
              border: `1px solid ${
                operation === "reclassify"
                  ? theme.colors.border.orangeStrong
                  : theme.colors.border.light
              }`,
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

        {/* Target (only for reclassify) */}
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
                style={{ display: "flex", flexDirection: "column", gap: theme.spacing[2] }}
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
                    style={{ background: theme.colors.gray[800], color: theme.colors.white }}
                  >
                    Choose category...
                  </option>
                  {categories.map((cat) => (
                    <option
                      key={cat.id}
                      value={cat.id}
                      style={{ background: theme.colors.gray[800], color: theme.colors.white }}
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
                      style={{ background: theme.colors.gray[800], color: theme.colors.white }}
                    >
                      Choose specific type...
                    </option>
                    {categories
                      .find((cat) => cat.id === targetCategory)
                      ?.items.map((item) => (
                        <option
                          key={item.id}
                          value={item.id}
                          style={{ background: theme.colors.gray[800], color: theme.colors.white }}
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
          disabled={operation === "reclassify" && (!targetCategory || !targetItem)}
          style={{
            ...styleUtils.buttonBase(),
            background:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? theme.colors.background.card
                : theme.colors.background.overlay,
            border: `1px solid ${
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? theme.colors.border.light
                : theme.colors.border.orangeStrong
            }`,
            color:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? theme.colors.gray[600]
                : theme.colors.primary.orangeLight,
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
