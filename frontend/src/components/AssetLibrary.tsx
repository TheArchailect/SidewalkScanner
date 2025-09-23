import type React from "react";

import { useState, useEffect, type RefObject } from "react";
import { useWebRpc } from "../hooks/useWebRpc";
import { theme, styleUtils } from "../theme";

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

    if (!Array.isArray(assets)) {
      return [{ id: "all", name: "All Assets", count: 0 }];
    }

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
  const calculatedCategories = calculateCategories(availableAssets || []);

  const filteredAssets = (availableAssets || []).filter((asset) => {
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
        right: theme.spacing[6],
        top: "70px",
        bottom: theme.spacing[6],
        width: "320px",
        ...styleUtils.glassPanel("dark"),
        zIndex: theme.zIndex.modal,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Asset Library Header */}
      <div
        style={{
          padding: theme.spacing[5],
          borderBottom: `1px solid ${theme.colors.border.default}`,
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
        }}
      >
        <h3
          style={{
            margin: 0,
            ...styleUtils.text.subtitle(),
          }}
        >
          Asset Library
          {isLoading && (
            <span
              style={{
                fontSize: theme.fontSizes.xs,
                color: theme.colors.gray[600],
                marginLeft: theme.spacing[3],
              }}
            >
              Loading...
            </span>
          )}
        </h3>
        <div style={{ display: "flex", gap: theme.spacing[1] }}>
          <button
            onClick={() => {
              setAssetViewMode("grid");
              returnFocusToCanvas();
            }}
            style={{
              background:
                assetViewMode === "grid"
                  ? theme.colors.background.hover
                  : "transparent",
              border: "none",
              color:
                assetViewMode === "grid"
                  ? theme.colors.white
                  : theme.colors.gray[600],
              padding: theme.spacing[1],
              borderRadius: theme.radius.sm,
              cursor: "pointer",
              transition: theme.transitions.fast,
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
                  ? theme.colors.background.hover
                  : "transparent",
              border: "none",
              color:
                assetViewMode === "list"
                  ? theme.colors.white
                  : theme.colors.gray[600],
              padding: theme.spacing[1],
              borderRadius: theme.radius.sm,
              cursor: "pointer",
              transition: theme.transitions.fast,
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
              color: theme.colors.gray[600],
              padding: theme.spacing[1],
              borderRadius: theme.radius.sm,
              cursor: "pointer",
              transition: theme.transitions.fast,
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
      <div style={{ padding: `${theme.spacing[4]} ${theme.spacing[5]}` }}>
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
            ...styleUtils.inputField(),
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            fontSize: theme.fontSizes.base,
            boxSizing: "border-box",
          }}
        />
      </div>

      {/* Category Tabs */}
      <div
        style={{
          padding: `0 ${theme.spacing[5]} ${theme.spacing[6]}`,
          display: "flex",
          flexWrap: "wrap",
          gap: theme.spacing[2],
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
              ...styleUtils.toolItem(selectedCategory === category.id),
              padding: `${theme.spacing[1]} ${theme.spacing[3]}`,
              borderRadius: theme.radius.xl,
              fontSize: theme.fontSizes.sm,
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
            margin: `0 ${theme.spacing[5]} ${theme.spacing[4]}`,
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            background: `rgba(255, 151, 0, 0.1)`,
            border: `1px solid ${theme.colors.border.orange}`,
            borderRadius: theme.radius.base,
            fontSize: theme.fontSizes.sm,
            color: theme.colors.primary.orange,
          }}
        >
          Selected: {selectedAsset.name}
          {selectedAsset.point_count && (
            <span
              style={{
                color: theme.colors.gray[600],
                marginLeft: theme.spacing[3],
              }}
            >
              ({selectedAsset.point_count.toLocaleString()} points)
            </span>
          )}
        </div>
      )}

      {/* Asset Grid/List */}
      <div
        style={{
          flex: 1,
          padding: `0 ${theme.spacing[5]} ${theme.spacing[5]}`,
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
              ...styleUtils.text.body(),
              color: theme.colors.gray[600],
            }}
          >
            {isLoading ? "Loading assets..." : "No assets found"}
          </div>
        ) : assetViewMode === "grid" ? (
          <div
            style={{
              display: "grid",
              gridTemplateColumns: "repeat(2, 1fr)",
              gap: theme.spacing[3],
            }}
          >
            {filteredAssets.map((asset) => (
              <div
                key={asset.id}
                onClick={() => handleAssetSelection(asset)}
                style={{
                  ...styleUtils.toolItem(selectedAsset?.id === asset.id),
                  padding: theme.spacing[3],
                  borderRadius: theme.radius.md,
                  cursor: "pointer",
                  transition: theme.transitions.fast,
                }}
                onMouseEnter={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background = `rgba(0, 104, 255, 0.08)`;
                    e.currentTarget.style.borderColor =
                      theme.colors.border.blue;
                  }
                }}
                onMouseLeave={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background =
                      theme.colors.background.input;
                    e.currentTarget.style.borderColor =
                      theme.colors.border.default;
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
                    borderRadius: theme.radius.base,
                    marginBottom: theme.spacing[2],
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: theme.fontSizes.xs,
                    color: theme.colors.gray[600],
                  }}
                >
                  3D Preview
                </div>
                <div
                  style={{
                    ...styleUtils.text.body(),
                    fontSize: theme.fontSizes.sm,
                    fontWeight: theme.fontWeights.medium,
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
                    ...styleUtils.text.caption(),
                    fontSize: theme.fontSizes.xs,
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
          <div
            style={{
              display: "flex",
              flexDirection: "column",
              gap: theme.spacing[1],
            }}
          >
            {filteredAssets.map((asset) => (
              <div
                key={asset.id}
                onClick={() => handleAssetSelection(asset)}
                style={{
                  ...styleUtils.toolItem(selectedAsset?.id === asset.id),
                  padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
                  borderRadius: theme.radius.base,
                  cursor: "pointer",
                  display: "flex",
                  alignItems: "center",
                  gap: theme.spacing[4],
                  transition: theme.transitions.fast,
                }}
                onMouseEnter={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background = `rgba(0, 104, 255, 0.08)`;
                  }
                }}
                onMouseLeave={(e) => {
                  if (selectedAsset?.id !== asset.id) {
                    e.currentTarget.style.background =
                      theme.colors.background.input;
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
                    borderRadius: theme.radius.sm,
                    flexShrink: 0,
                    display: "flex",
                    alignItems: "center",
                    justifyContent: "center",
                    fontSize: "8px",
                    color: theme.colors.gray[600],
                  }}
                >
                  3D
                </div>
                <div style={{ flex: 1, minWidth: 0 }}>
                  <div
                    style={{
                      ...styleUtils.text.body(),
                      fontSize: theme.fontSizes.base,
                      fontWeight: theme.fontWeights.medium,
                      overflow: "hidden",
                      textOverflow: "ellipsis",
                      whiteSpace: "nowrap",
                    }}
                  >
                    {asset.name}
                  </div>
                  <div
                    style={{
                      ...styleUtils.text.caption(),
                      fontSize: theme.fontSizes.xs,
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
