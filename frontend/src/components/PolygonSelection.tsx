"use client";

import type React from "react";
import { useState, type RefObject } from "react";

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
      color: "#3b82f6",
      items: [
        { id: "car", name: "Cars", selected: false },
        { id: "truck", name: "Trucks", selected: false },
        { id: "bike", name: "Bikes", selected: false },
      ],
    },
    {
      id: "vegetation",
      name: "Vegetation",
      color: "#22c55e",
      items: [
        { id: "tree", name: "Trees", selected: false },
        { id: "grass", name: "Grass", selected: false },
        { id: "bush", name: "Bushes", selected: false },
      ],
    },
    {
      id: "infrastructure",
      name: "Infrastructure",
      color: "#f59e0b",
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

  if (!isVisible) return null;

  return (
    <div
      style={{
        position: "fixed",
        right: "20px",
        top: "70px",
        width: "360px",
        background: "rgba(0, 0, 0, 0.4)",
        backdropFilter: "blur(20px)",
        borderRadius: "8px",
        border: "1px solid rgba(255, 255, 255, 0.08)",
        zIndex: 15,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: "16px",
          borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
        }}
      >
        <h3
          style={{
            margin: "0 0 16px 0",
            fontSize: "14px",
            fontWeight: "600",
            color: "#fff",
            textAlign: "center",
          }}
        >
          Polygon Tool
        </h3>

        <div style={{ display: "flex", gap: "6px", marginBottom: "12px" }}>
          <button
            onClick={() => {
              setOperation("hide");
              returnFocusToCanvas();
            }}
            style={{
              background:
                operation === "hide"
                  ? "rgba(0, 0, 0, 0.6)"
                  : "rgba(0, 0, 0, 0.3)",
              border: `1px solid ${operation === "hide" ? "rgba(239, 68, 68, 0.8)" : "rgba(255, 255, 255, 0.1)"}`,
              color: operation === "hide" ? "#fca5a5" : "#999",
              padding: "8px 12px",
              borderRadius: "6px",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: "500",
              transition: "all 0.15s ease",
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
              background:
                operation === "reclassify"
                  ? "rgba(0, 0, 0, 0.6)"
                  : "rgba(0, 0, 0, 0.3)",
              border: `1px solid ${operation === "reclassify" ? "rgba(59, 130, 246, 0.8)" : "rgba(255, 255, 255, 0.1)"}`,
              color: operation === "reclassify" ? "#93c5fd" : "#999",
              padding: "8px 12px",
              borderRadius: "6px",
              cursor: "pointer",
              fontSize: "11px",
              fontWeight: "500",
              transition: "all 0.15s ease",
              flex: 1,
            }}
          >
            Reclassify Points
          </button>
        </div>

        <div
          style={{
            fontSize: "11px",
            color: "#999",
            textAlign: "center",
            lineHeight: "1.4",
            background: "rgba(0, 0, 0, 0.3)",
            padding: "8px 12px",
            borderRadius: "4px",
            border: "1px solid rgba(255, 255, 255, 0.1)",
          }}
        >
          {operation === "hide"
            ? "Choose which types to hide, or leave empty to hide all points in the polygon"
            : "Choose which types to reclassify, or leave empty to reclassify all points in the polygon"}
        </div>
      </div>

      <div
        style={{
          padding: "12px 16px",
          flex: 1,
          maxHeight: "400px",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: "12px",
        }}
      >
        {/* Source Selection Card */}
        <div
          style={{
            background: "rgba(0, 0, 0, 0.3)",
            border: "1px solid rgba(255, 255, 255, 0.08)",
            borderRadius: "6px",
            padding: "12px",
          }}
        >
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
              marginBottom: "12px",
            }}
          >
            <h4
              style={{
                margin: 0,
                fontSize: "12px",
                fontWeight: "600",
                color: "#fff",
              }}
            >
              {operation === "hide" ? "What to Hide" : "What to Reclassify"}
            </h4>

            <div
              style={{
                fontSize: "10px",
                padding: "2px 6px",
                borderRadius: "8px",
                fontWeight: "500",
                background: hasAnySelection
                  ? "rgba(0, 255, 136, 0.2)"
                  : "rgba(0, 0, 0, 0.4)",
                color: hasAnySelection ? "#00ff88" : "#999",
                border: `1px solid ${hasAnySelection ? "rgba(0, 255, 136, 0.3)" : "rgba(255, 255, 255, 0.1)"}`,
              }}
            >
              {hasAnySelection ? `${getSelectedCount()} selected` : "All types"}
            </div>
          </div>

          {categories.map((category) => (
            <div key={category.id} style={{ marginBottom: "8px" }}>
              <div
                style={{
                  fontSize: "11px",
                  color: category.color,
                  marginBottom: "4px",
                  fontWeight: "500",
                  display: "flex",
                  alignItems: "center",
                  gap: "4px",
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
                  gap: "4px",
                  paddingLeft: "10px",
                }}
              >
                {category.items.map((item) => (
                  <button
                    key={item.id}
                    onClick={() => toggleSourceItem(category.id, item.id)}
                    style={{
                      background: item.selected
                        ? "rgba(0, 255, 136, 0.2)"
                        : "rgba(0, 0, 0, 0.3)",
                      border: `1px solid ${item.selected ? "rgba(0, 255, 136, 0.4)" : "rgba(255, 255, 255, 0.1)"}`,
                      borderRadius: "4px",
                      padding: "4px 8px",
                      cursor: "pointer",
                      fontSize: "10px",
                      color: item.selected ? "#00ff88" : "#999",
                      fontWeight: item.selected ? "500" : "400",
                      transition: "all 0.15s ease",
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
            transition: "all 0.3s ease",
          }}
        >
          {operation === "reclassify" && (
            <div
              style={{
                background: "rgba(59, 130, 246, 0.1)",
                border: "1px solid rgba(59, 130, 246, 0.2)",
                borderRadius: "6px",
                padding: "12px",
              }}
            >
              <h4
                style={{
                  margin: "0 0 8px 0",
                  fontSize: "12px",
                  fontWeight: "600",
                  color: "#93c5fd",
                  display: "flex",
                  alignItems: "center",
                  gap: "6px",
                }}
              >
                <span>→</span>
                Reclassify To
              </h4>

              <div
                style={{ display: "flex", flexDirection: "column", gap: "6px" }}
              >
                <select
                  value={targetCategory}
                  onChange={(e) => {
                    setTargetCategory(e.target.value);
                    setTargetItem("");
                    returnFocusToCanvas();
                  }}
                  style={{
                    background: "rgba(0, 0, 0, 0.4)",
                    border: "1px solid rgba(255, 255, 255, 0.1)",
                    borderRadius: "4px",
                    color: "#fff",
                    padding: "8px 10px",
                    fontSize: "11px",
                    cursor: "pointer",
                    outline: "none",
                  }}
                >
                  <option
                    value=""
                    style={{ background: "#1a1a1a", color: "#fff" }}
                  >
                    Choose category...
                  </option>
                  {categories.map((cat) => (
                    <option
                      key={cat.id}
                      value={cat.id}
                      style={{ background: "#1a1a1a", color: "#fff" }}
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
                      background: "rgba(0, 0, 0, 0.4)",
                      border: "1px solid rgba(255, 255, 255, 0.1)",
                      borderRadius: "4px",
                      color: "#fff",
                      padding: "8px 10px",
                      fontSize: "11px",
                      cursor: "pointer",
                      outline: "none",
                    }}
                  >
                    <option
                      value=""
                      style={{ background: "#1a1a1a", color: "#fff" }}
                    >
                      Choose specific type...
                    </option>
                    {categories
                      .find((cat) => cat.id === targetCategory)
                      ?.items.map((item) => (
                        <option
                          key={item.id}
                          value={item.id}
                          style={{ background: "#1a1a1a", color: "#fff" }}
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
          padding: "12px 16px",
          borderTop: "1px solid rgba(255, 255, 255, 0.08)",
          display: "flex",
          gap: "8px",
        }}
      >
        <button
          onClick={() => console.log("cancel")}
          style={{
            background: "rgba(0, 0, 0, 0.3)",
            border: "1px solid rgba(255, 255, 255, 0.1)",
            borderRadius: "4px",
            color: "#999",
            padding: "8px 12px",
            fontSize: "11px",
            fontWeight: "500",
            cursor: "pointer",
            transition: "all 0.15s ease",
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
            background:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? "rgba(0, 0, 0, 0.3)"
                : operation === "hide"
                  ? "rgba(0, 0, 0, 0.6)"
                  : "rgba(0, 0, 0, 0.6)",
            border: `1px solid ${
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? "rgba(255, 255, 255, 0.1)"
                : operation === "hide"
                  ? "rgba(239, 68, 68, 0.8)"
                  : "rgba(59, 130, 246, 0.8)"
            }`,
            borderRadius: "4px",
            color:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? "#666"
                : operation === "hide"
                  ? "#fca5a5"
                  : "#93c5fd",
            padding: "8px 12px",
            fontSize: "11px",
            fontWeight: "600",
            cursor:
              operation === "reclassify" && (!targetCategory || !targetItem)
                ? "not-allowed"
                : "pointer",
            transition: "all 0.15s ease",
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
