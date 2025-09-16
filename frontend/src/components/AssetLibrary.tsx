"use client";

import { useState, RefObject } from "react";

interface AssetLibraryProps {
  isVisible: boolean;
  onClose: () => void;
  canvasRef: RefObject<HTMLElement>;
}

interface AssetCategory {
  id: string;
  name: string;
  count: number;
}

interface Asset {
  id: string;
  name: string;
  category: string;
  type: string;
}

type ViewMode = "grid" | "list";

const AssetLibrary: React.FC<AssetLibraryProps> = ({
  isVisible,
  onClose,
  canvasRef,
}) => {
  const [assetViewMode, setAssetViewMode] = useState<ViewMode>("grid");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState<string>("");

  const returnFocusToCanvas = (): void => {
    setTimeout(() => {
      if (canvasRef.current) {
        canvasRef.current.focus();
      }
    }, 100);
  };

  const assetCategories: AssetCategory[] = [
    { id: "all", name: "All Assets", count: 10 },
    { id: "vehicles", name: "Vehicles", count: 2 },
    { id: "street_furniture", name: "Street Furniture", count: 4 },
    { id: "vegetation", name: "Vegetation", count: 4 },
  ];

  const mockAssets: Asset[] = [
    // Vehicles
    {
      id: "vehicle-001",
      name: "Maintenance Truck",
      category: "vehicles",
      type: "Municipal Vehicle",
    },
    {
      id: "vehicle-002",
      name: "Street Sweeper",
      category: "vehicles",
      type: "Cleaning Equipment",
    },

    // Street Furniture
    {
      id: "furniture-001",
      name: "Bus Stop Shelter",
      category: "street_furniture",
      type: "Public Shelter",
    },
    {
      id: "furniture-002",
      name: "Park Bench",
      category: "street_furniture",
      type: "Seating",
    },
    {
      id: "furniture-003",
      name: "Street Light",
      category: "street_furniture",
      type: "Lighting",
    },
    {
      id: "furniture-004",
      name: "Traffic Island",
      category: "street_furniture",
      type: "Traffic Control",
    },

    // Vegetation
    {
      id: "vegetation-001",
      name: "River Red Gum",
      category: "vegetation",
      type: "Tree",
    },
    {
      id: "vegetation-002",
      name: "Lavender Garden Bed",
      category: "vegetation",
      type: "Garden Bed",
    },
    {
      id: "vegetation-003",
      name: "Plane Tree Avenue",
      category: "vegetation",
      type: "Tree Line",
    },
    {
      id: "vegetation-004",
      name: "Native Grass Verge",
      category: "vegetation",
      type: "Ground Cover",
    },
  ];

  const filteredAssets = mockAssets.filter((asset) => {
    const matchesCategory =
      selectedCategory === "all" || asset.category === selectedCategory;
    const matchesSearch = asset.name
      .toLowerCase()
      .includes(searchQuery.toLowerCase());
    return matchesCategory && matchesSearch;
  });

  if (!isVisible) return null;

  return (
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
          padding: "0 16px 20px",
          display: "flex",
          flexWrap: "wrap",
          gap: "6px",
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
          <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
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
  );
};

export default AssetLibrary;
