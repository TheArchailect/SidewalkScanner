"use client";

import { useState, useEffect, RefObject } from "react";
import { useWebRpc } from "../hooks/useWebRpc";

interface AssetLibraryProps {
  isVisible: boolean;
  canvasRef: RefObject<HTMLIFrameElement | null>;
}

interface AssetCategory {
  id: string;
  name: string;
  count: number;
}

interface Asset {
  id: string;
  name: string;
  category?: string;
  type?: string;
  point_count?: number;
  uv_bounds?: {
    uv_min: number[];
    uv_max: number[];
  };
  local_bounds?: {
    min_x: number;
    min_y: number;
    min_z: number;
    max_x: number;
    max_y: number;
    max_z: number;
  };
}

type ViewMode = "grid" | "list";

const AssetLibrary: React.FC<AssetLibraryProps> = ({
  isVisible,
  canvasRef,
}) => {
  const {
    availableAssets,
    selectedAsset,
    selectAsset,
    getAvailableAssets,
    selectTool,
  } = useWebRpc(canvasRef);

  const [assetViewMode, setAssetViewMode] = useState<ViewMode>("grid");
  const [selectedCategory, setSelectedCategory] = useState<string>("all");
  const [searchQuery, setSearchQuery] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);

  useEffect(() => {
    if (isVisible) {
      loadAssetData();
      selectTool("assets").catch(console.error);
    }
  }, [isVisible]);

  const loadAssetData = async () => {
    setIsLoading(true);
    try {
      await getAvailableAssets();
    } catch (error) {
      console.error("Failed to load asset data:", error);
    } finally {
      setIsLoading(false);
    }
  };

  const returnFocusToCanvas = (): void => {
    setTimeout(() => {
      if (canvasRef.current) {
        canvasRef.current.focus();
      }
    }, 100);
  };

  const handleAssetSelection = async (asset: Asset) => {
    try {
      await selectAsset(asset.id);
      returnFocusToCanvas();
    } catch (error) {
      console.error("Failed to select asset:", error);
    }
  };

  // Calculate categories dynamically from assets
  const calculateCategories = (assets: Asset[]): AssetCategory[] => {
    const categoryMap = new Map<string, number>();

    // Count assets by category
    assets.forEach((asset) => {
      const category = asset.category || "uncategorized";
      categoryMap.set(category, (categoryMap.get(category) || 0) + 1);
    });

    // Convert to category array
    const categories: AssetCategory[] = [
      { id: "all", name: "All Assets", count: assets.length },
    ];

    // Add discovered categories
    categoryMap.forEach((count, categoryId) => {
      const categoryName =
        categoryId.charAt(0).toUpperCase() + categoryId.slice(1);
      categories.push({
        id: categoryId,
        name: categoryName,
        count: count,
      });
    });

    return categories;
  };

  // Use calculated categories
  const calculatedCategories = calculateCategories(availableAssets);

  const filteredAssets = availableAssets.filter((asset) => {
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
          {isLoading && (
            <span
              style={{ fontSize: "10px", color: "#666", marginLeft: "8px" }}
            >
              Loading...
            </span>
          )}
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
          <button
            onClick={() => {
              loadAssetData();
              returnFocusToCanvas();
            }}
            style={{
              background: "transparent",
              border: "none",
              color: "#666",
              padding: "4px",
              borderRadius: "3px",
              cursor: "pointer",
            }}
            title="Refresh assets"
          >
            <svg
              width="14"
              height="14"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth="1.5"
            >
              <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
              <path d="M21 3v5h-5" />
              <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16" />
              <path d="M3 21v-5h5" />
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
        {calculatedCategories.map((category) => (
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

      {/* Selected Asset Info */}
      {selectedAsset && (
        <div
          style={{
            margin: "0 16px 12px",
            padding: "8px 12px",
            background: "rgba(0, 255, 136, 0.1)",
            border: "1px solid rgba(0, 255, 136, 0.3)",
            borderRadius: "4px",
            fontSize: "11px",
            color: "#00ff88",
          }}
        >
          Selected: {selectedAsset.name}
          {selectedAsset.point_count && (
            <span style={{ color: "#666", marginLeft: "8px" }}>
              ({selectedAsset.point_count.toLocaleString()} points)
            </span>
          )}
        </div>
      )}

      {/* Asset Grid/List */}
      <div
        style={{
          flex: 1,
          padding: "0 16px 16px",
          overflowY: "auto",
        }}
      >
        {filteredAssets.length === 0 ? (
          <div
            style={{
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              height: "100%",
              color: "#666",
              fontSize: "12px",
            }}
          >
            {isLoading ? "Loading assets..." : "No assets found"}
          </div>
        ) : assetViewMode === "grid" ? (
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
                onClick={() => handleAssetSelection(asset)}
                style={{
                  background:
                    selectedAsset?.id === asset.id
                      ? "rgba(0, 255, 136, 0.1)"
                      : "rgba(255, 255, 255, 0.05)",
                  border:
                    selectedAsset?.id === asset.id
                      ? "1px solid rgba(0, 255, 136, 0.3)"
                      : "1px solid rgba(255, 255, 255, 0.08)",
                  borderRadius: "6px",
                  padding: "8px",
                  cursor: "pointer",
                  transition: "all 0.15s ease",
                }}
                onMouseEnter={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background =
                      "rgba(255, 255, 255, 0.08)";
                    e.currentTarget.style.borderColor =
                      "rgba(255, 255, 255, 0.15)";
                  }
                }}
                onMouseLeave={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background =
                      "rgba(255, 255, 255, 0.05)";
                    e.currentTarget.style.borderColor =
                      "rgba(255, 255, 255, 0.08)";
                  }
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
                  {asset.point_count
                    ? `${asset.point_count.toLocaleString()} points`
                    : "Asset"}
                </div>
              </div>
            ))}
          </div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "4px" }}>
            {filteredAssets.map((asset) => (
              <div
                key={asset.id}
                onClick={() => handleAssetSelection(asset)}
                style={{
                  background:
                    selectedAsset?.id === asset.id
                      ? "rgba(0, 255, 136, 0.1)"
                      : "rgba(255, 255, 255, 0.05)",
                  border:
                    selectedAsset?.id === asset.id
                      ? "1px solid rgba(0, 255, 136, 0.3)"
                      : "1px solid rgba(255, 255, 255, 0.08)",
                  borderRadius: "4px",
                  padding: "8px 12px",
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  gap: "12px",
                  transition: "all 0.15s ease",
                }}
                onMouseEnter={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background =
                      "rgba(255, 255, 255, 0.08)";
                  }
                }}
                onMouseLeave={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background =
                      "rgba(255, 255, 255, 0.05)";
                  }
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
                    {asset.point_count
                      ? `${asset.point_count.toLocaleString()} points`
                      : "Asset"}
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
