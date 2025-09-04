"use client";

import { useState, useRef } from "react";

const ScannerApp = () => {
  const [selectedTool, setSelectedTool] = useState("polygon");
  const [showAssetLibrary, setShowAssetLibrary] = useState(false);
  const [assetViewMode, setAssetViewMode] = useState("grid");
  const [selectedCategory, setSelectedCategory] = useState("all");
  const [searchQuery, setSearchQuery] = useState("");
  const canvasRef = useRef(null);

  const returnFocusToCanvas = () => {
    setTimeout(() => {
      if (canvasRef.current) {
        canvasRef.current.focus();
      }
    }, 100);
  };

  const handleToolSelect = (toolId) => {
    setSelectedTool(toolId);
    if (toolId === "assets") {
      setShowAssetLibrary(!showAssetLibrary);
    }
    returnFocusToCanvas();
  };

  const assetCategories = [
    { id: "all", name: "All Assets", count: 24 },
    { id: "buildings", name: "Buildings", count: 8 },
    { id: "vehicles", name: "Vehicles", count: 6 },
    { id: "furniture", name: "Furniture", count: 5 },
    { id: "nature", name: "Nature", count: 5 },
  ];

  const mockAssets = [
    {
      id: 1,
      name: "Modern Building A",
      category: "buildings",
      type: "3D Model",
    },
    { id: 2, name: "Sports Car", category: "vehicles", type: "3D Model" },
    { id: 3, name: "Office Chair", category: "furniture", type: "3D Model" },
    { id: 4, name: "Pine Tree", category: "nature", type: "3D Model" },
    {
      id: 5,
      name: "Apartment Complex",
      category: "buildings",
      type: "3D Model",
    },
    { id: 6, name: "Delivery Truck", category: "vehicles", type: "3D Model" },
    {
      id: 7,
      name: "Conference Table",
      category: "furniture",
      type: "3D Model",
    },
    { id: 8, name: "Oak Tree", category: "nature", type: "3D Model" },
  ];

  const filteredAssets = mockAssets.filter((asset) => {
    const matchesCategory =
      selectedCategory === "all" || asset.category === selectedCategory;
    const matchesSearch = asset.name
      .toLowerCase()
      .includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

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
          {
            id: "assets",
            icon: (
              <svg
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="1.5"
              >
                <rect x="3" y="3" width="7" height="7" />
                <rect x="14" y="3" width="7" height="7" />
                <rect x="14" y="14" width="7" height="7" />
                <rect x="3" y="14" width="7" height="7" />
              </svg>
            ),
          },
        ].map((tool, index) => (
          <button
            key={tool.id}
            onClick={() => handleToolSelect(tool.id)}
            style={{
              width: "36px",
              height: "36px",
              border: "none",
              background:
                selectedTool === tool.id ||
                (tool.id === "assets" && showAssetLibrary)
                  ? "rgba(255, 255, 255, 0.15)"
                  : "transparent",
              borderRadius: "4px",
              color:
                selectedTool === tool.id ||
                (tool.id === "assets" && showAssetLibrary)
                  ? "#fff"
                  : "#666",
              cursor: "pointer",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              transition: "all 0.15s ease",
              marginBottom: index < 2 ? "4px" : "0",
              position: "relative",
            }}
            onMouseEnter={(e) => {
              if (
                selectedTool !== tool.id &&
                !(tool.id === "assets" && showAssetLibrary)
              ) {
                e.currentTarget.style.background = "rgba(255, 255, 255, 0.08)";
                e.currentTarget.style.color = "#aaa";
              }
            }}
            onMouseLeave={(e) => {
              if (
                selectedTool !== tool.id &&
                !(tool.id === "assets" && showAssetLibrary)
              ) {
                e.currentTarget.style.background = "transparent";
                e.currentTarget.style.color = "#666";
              }
            }}
          >
            {tool.icon}
            {(selectedTool === tool.id ||
              (tool.id === "assets" && showAssetLibrary)) && (
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

      {/* Asset Library Panel */}
      {showAssetLibrary && (
        <div
          style={{
            position: "fixed",
            right: "20px",
            top: "70px",
            bottom: "20px",
            width: "320px",
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
          {/* Asset Library Header */}
          <div
            style={{
              padding: "16px",
              borderBottom: "1px solid rgba(255, 255, 255, 0.08)",
              display: "flex",
              alignItems: "center",
              justifyContent: "space-between",
            }}
          >
            <h3
              style={{
                margin: 0,
                fontSize: "14px",
                fontWeight: "600",
                color: "#fff",
              }}
            >
              Asset Library
            </h3>
            <div style={{ display: "flex", gap: "4px" }}>
              <button
                onClick={() => {
                  setAssetViewMode("grid");
                  returnFocusToCanvas();
                }}
                style={{
                  background:
                    assetViewMode === "grid"
                      ? "rgba(255, 255, 255, 0.15)"
                      : "transparent",
                  border: "none",
                  color: assetViewMode === "grid" ? "#fff" : "#666",
                  padding: "4px",
                  borderRadius: "3px",
                  cursor: "pointer",
                }}
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                >
                  <rect x="3" y="3" width="7" height="7" />
                  <rect x="14" y="3" width="7" height="7" />
                  <rect x="14" y="14" width="7" height="7" />
                  <rect x="3" y="14" width="7" height="7" />
                </svg>
              </button>
              <button
                onClick={() => {
                  setAssetViewMode("list");
                  returnFocusToCanvas();
                }}
                style={{
                  background:
                    assetViewMode === "list"
                      ? "rgba(255, 255, 255, 0.15)"
                      : "transparent",
                  border: "none",
                  color: assetViewMode === "list" ? "#fff" : "#666",
                  padding: "4px",
                  borderRadius: "3px",
                  cursor: "pointer",
                }}
              >
                <svg
                  width="14"
                  height="14"
                  viewBox="0 0 24 24"
                  fill="none"
                  stroke="currentColor"
                  strokeWidth="1.5"
                >
                  <line x1="8" y1="6" x2="21" y2="6" />
                  <line x1="8" y1="12" x2="21" y2="12" />
                  <line x1="8" y1="18" x2="21" y2="18" />
                  <line x1="3" y1="6" x2="3.01" y2="6" />
                  <line x1="3" y1="12" x2="3.01" y2="12" />
                  <line x1="3" y1="18" x2="3.01" y2="18" />
                </svg>
              </button>
            </div>
          </div>

          {/* Search Bar */}
          <div style={{ padding: "12px 16px" }}>
            <input
              type="text"
              placeholder="Search assets..."
              value={searchQuery}
              onChange={(e) => {
                setSearchQuery(e.target.value);
                returnFocusToCanvas();
              }}
              style={{
                width: "100%",
                background: "rgba(255, 255, 255, 0.05)",
                border: "1px solid rgba(255, 255, 255, 0.1)",
                borderRadius: "4px",
                padding: "8px 12px",
                fontSize: "12px",
                color: "#fff",
                outline: "none",
                boxSizing: "border-box",
              }}
            />
          </div>

          {/* Category Tabs */}
          <div
            style={{
              padding: "0 16px 12px",
              display: "flex",
              gap: "4px",
              overflowX: "auto",
            }}
          >
            {assetCategories.map((category) => (
              <button
                key={category.id}
                onClick={() => {
                  setSelectedCategory(category.id);
                  returnFocusToCanvas();
                }}
                style={{
                  background:
                    selectedCategory === category.id
                      ? "rgba(0, 255, 136, 0.2)"
                      : "rgba(255, 255, 255, 0.05)",
                  border: "1px solid rgba(255, 255, 255, 0.1)",
                  borderRadius: "12px",
                  color: selectedCategory === category.id ? "#00ff88" : "#999",
                  padding: "4px 8px",
                  fontSize: "11px",
                  cursor: "pointer",
                  whiteSpace: "nowrap",
                  transition: "all 0.15s ease",
                }}
              >
                {category.name} ({category.count})
              </button>
            ))}
          </div>

          {/* Asset Grid/List */}
          <div
            style={{
              flex: 1,
              padding: "0 16px 16px",
              overflowY: "auto",
            }}
          >
            {assetViewMode === "grid" ? (
              <div
                style={{
                  display: "grid",
                  gridTemplateColumns: "repeat(2, 1fr)",
                  gap: "8px",
                }}
              >
                {filteredAssets.map((asset) => (
                  <div
                    key={asset.id}
                    onClick={() => {
                      console.log("[v0] Asset selected:", asset.name);
                      returnFocusToCanvas();
                    }}
                    style={{
                      background: "rgba(255, 255, 255, 0.05)",
                      border: "1px solid rgba(255, 255, 255, 0.08)",
                      borderRadius: "6px",
                      padding: "8px",
                      cursor: "pointer",
                      transition: "all 0.15s ease",
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background =
                        "rgba(255, 255, 255, 0.08)";
                      e.currentTarget.style.borderColor =
                        "rgba(255, 255, 255, 0.15)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background =
                        "rgba(255, 255, 255, 0.05)";
                      e.currentTarget.style.borderColor =
                        "rgba(255, 255, 255, 0.08)";
                    }}
                  >
                    <div
                      style={{
                        width: "100%",
                        height: "80px",
                        background:
                          "linear-gradient(135deg, rgba(255,255,255,0.1) 0%, rgba(255,255,255,0.05) 100%)",
                        backgroundSize: "cover",
                        backgroundPosition: "center",
                        borderRadius: "4px",
                        marginBottom: "6px",
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        fontSize: "10px",
                        color: "#666",
                      }}
                    >
                      3D Preview
                    </div>
                    <div
                      style={{
                        fontSize: "11px",
                        color: "#fff",
                        fontWeight: "500",
                        marginBottom: "2px",
                        overflow: "hidden",
                        textOverflow: "ellipsis",
                        whiteSpace: "nowrap",
                      }}
                    >
                      {asset.name}
                    </div>
                    <div
                      style={{
                        fontSize: "10px",
                        color: "#666",
                      }}
                    >
                      {asset.type}
                    </div>
                  </div>
                ))}
              </div>
            ) : (
              <div
                style={{ display: "flex", flexDirection: "column", gap: "4px" }}
              >
                {filteredAssets.map((asset) => (
                  <div
                    key={asset.id}
                    onClick={() => {
                      console.log("[v0] Asset selected:", asset.name);
                      returnFocusToCanvas();
                    }}
                    style={{
                      background: "rgba(255, 255, 255, 0.05)",
                      border: "1px solid rgba(255, 255, 255, 0.08)",
                      borderRadius: "4px",
                      padding: "8px 12px",
                      cursor: "pointer",
                      display: "flex",
                      alignItems: "center",
                      gap: "12px",
                      transition: "all 0.15s ease",
                    }}
                    onMouseEnter={(e) => {
                      e.currentTarget.style.background =
                        "rgba(255, 255, 255, 0.08)";
                    }}
                    onMouseLeave={(e) => {
                      e.currentTarget.style.background =
                        "rgba(255, 255, 255, 0.05)";
                    }}
                  >
                    <div
                      style={{
                        width: "32px",
                        height: "32px",
                        background:
                          "linear-gradient(135deg, rgba(255,255,255,0.1) 0%, rgba(255,255,255,0.05) 100%)",
                        backgroundSize: "cover",
                        backgroundPosition: "center",
                        borderRadius: "3px",
                        flexShrink: 0,
                        display: "flex",
                        alignItems: "center",
                        justifyContent: "center",
                        fontSize: "8px",
                        color: "#666",
                      }}
                    >
                      3D
                    </div>
                    <div style={{ flex: 1, minWidth: 0 }}>
                      <div
                        style={{
                          fontSize: "12px",
                          color: "#fff",
                          fontWeight: "500",
                          overflow: "hidden",
                          textOverflow: "ellipsis",
                          whiteSpace: "nowrap",
                        }}
                      >
                        {asset.name}
                      </div>
                      <div
                        style={{
                          fontSize: "10px",
                          color: "#666",
                        }}
                      >
                        {asset.type}
                      </div>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};

export default ScannerApp;
