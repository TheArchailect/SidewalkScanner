"use client";

import { useState, useEffect, useCallback, useRef, RefObject } from "react";

interface JsonRpcRequest {
  jsonrpc: "2.0";
  method: string;
  params?: Record<string, any>;
  id: number;
}

interface JsonRpcNotification {
  jsonrpc: "2.0";
  method: string;
  params?: Record<string, any>;
}

interface JsonRpcResponse {
  jsonrpc: "2.0";
  id: number;
  result?: any;
  error?: {
    code: number;
    message: string;
  };
}

interface JsonRpcMessage {
  jsonrpc: "2.0";
  method?: string;
  params?: Record<string, any>;
  id?: number;
  result?: any;
  error?: {
    code: number;
    message: string;
  };
}

interface PendingRequest {
  resolve: (value: any) => void;
  reject: (reason: any) => void;
}

interface Asset {
  id: string;
  name: string;
  category?: string;
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
  [key: string]: any;
}

interface Position3D {
  x: number;
  y: number;
  z: number;
}

interface PlacedAsset {
  id: number;
  asset: Asset;
  position: Position3D;
  [key: string]: any;
}

interface NotificationHandler {
  (params?: Record<string, any>): void;
}

export const useWebRpc = (canvasRef: RefObject<HTMLIFrameElement | null>) => {
  const [fps, setFps] = useState<number>(0);
  const [isConnected, setIsConnected] = useState<boolean>(false);
  const [availableAssets, setAvailableAssets] = useState<Asset[]>([]);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);
  const [placedAssets, setPlacedAssets] = useState<PlacedAsset[]>([]);

  const requestIdCounter = useRef<number>(1);
  const pendingRequests = useRef<Map<number, PendingRequest>>(new Map());
  const notificationHandlers = useRef<Map<string, NotificationHandler>>(
    new Map(),
  );

  // Generate unique request ID
  const generateRequestId = useCallback((): number => {
    return requestIdCounter.current++;
  }, []);

  // Send JSON-RPC request (expects response)
  const sendRequest = useCallback(
    <T = any,>(
      method: string,
      params: Record<string, any> = {},
    ): Promise<T> => {
      return new Promise<T>((resolve, reject) => {
        if (!canvasRef.current?.contentWindow) {
          console.log(
            `[DEBUG] ${method} failed - iframe:${!!canvasRef.current} contentWindow:${!!canvasRef.current?.contentWindow}`,
          );
          reject(new Error("Canvas not ready"));
          return;
        }

        const id = generateRequestId();
        const request: JsonRpcRequest = {
          jsonrpc: "2.0",
          method,
          params,
          id,
        };

        // Store pending request
        pendingRequests.current.set(id, { resolve, reject });

        // Send to WASM
        try {
          canvasRef.current.contentWindow.postMessage(
            JSON.stringify(request),
            "*",
          );
        } catch (error) {
          pendingRequests.current.delete(id);
          reject(error);
        }
      });
    },
    [generateRequestId, canvasRef],
  );

  // Send JSON-RPC notification (no response expected)
  const sendNotification = useCallback(
    (method: string, params: Record<string, any> = {}): void => {
      if (!canvasRef.current?.contentWindow) {
        return;
      }

      const notification: JsonRpcNotification = {
        jsonrpc: "2.0",
        method,
        params,
      };

      try {
        canvasRef.current.contentWindow.postMessage(
          JSON.stringify(notification),
          "*",
        );
      } catch (error) {
        console.error("Failed to send notification:", error);
      }
    },
    [canvasRef],
  );

  // Register handler for incoming notifications
  const onNotification = useCallback(
    (method: string, handler: NotificationHandler): void => {
      notificationHandlers.current.set(method, handler);
    },
    [],
  );

  // Handle incoming messages from Bevy
  useEffect(() => {
    const handleMessage = (event: MessageEvent): void => {
      // Ensure message is from our iframe
      if (
        canvasRef.current &&
        event.source !== canvasRef.current.contentWindow
      ) {
        return;
      }

      try {
        const message: JsonRpcMessage = JSON.parse(event.data as string);

        if (message.method === "debug_message") {
          // Uncomment for debugging: console.log("[RUST DEBUG]", message.params?.message);
        }

        // Handle JSON-RPC response
        if (message.jsonrpc === "2.0" && message.id !== undefined) {
          const pending = pendingRequests.current.get(message.id);
          if (pending) {
            pendingRequests.current.delete(message.id);

            if (message.error) {
              pending.reject(
                new Error(
                  `RPC Error ${message.error.code}: ${message.error.message}`,
                ),
              );
            } else {
              pending.resolve(message.result);
            }
          }
        }
        // Handle JSON-RPC notification
        else if (message.jsonrpc === "2.0" && message.method) {
          // Built-in handlers
          if (message.method === "fps_update") {
            setFps(message.params?.fps || 0);
          }

          // Asset-related notifications
          if (message.method === "asset_selected") {
            console.log("Asset selected:", message.params);
          }

          if (message.method === "tool_state_changed") {
            console.log("Tool state changed:", message.params);
          }

          // Custom handlers
          const handler = notificationHandlers.current.get(message.method);
          if (handler) {
            handler(message.params);
          }
        }
      } catch (error) {
        // Ignore parse errors from non-RPC messages
      }
    };

    window.addEventListener("message", handleMessage);
    return () => window.removeEventListener("message", handleMessage);
  }, [canvasRef]);

  // Monitor iframe connection state
  useEffect(() => {
    if (!canvasRef) return;

    const checkConnection = (): void => {
      if (canvasRef.current?.contentWindow) {
        setIsConnected(true);
      } else {
        setIsConnected(false);
      }
    };

    const interval = setInterval(checkConnection, 1000);
    checkConnection();

    return () => clearInterval(interval);
  }, [canvasRef]);

  // Specific method helpers
  const selectTool = useCallback(
    async (tool: string): Promise<any> => {
      try {
        const result = await sendRequest("tool_selection", { tool });
        return result;
      } catch (error) {
        console.error("Tool selection failed:", error);
        throw error;
      }
    },
    [sendRequest],
  );

  const setRenderMode = useCallback(
    async (mode: string): Promise<void> => {
      try {
        sendNotification("render_mode_changed", { mode });
      } catch (error) {
        console.error("Render mode change failed:", error);
        throw error;
      }
    },
    [sendNotification],
  );

  const getFps = useCallback(async (): Promise<number> => {
    try {
      const result = await sendRequest<{ fps: number }>("get_fps");
      return result.fps;
    } catch (error) {
      return 0;
    }
  }, [sendRequest]);

  // Selects an asset by ID and updates local state
  const selectAsset = useCallback(
    async (assetId: string): Promise<Asset | undefined> => {
      try {
        const result = await sendRequest<Asset>("select_asset", {
          asset_id: assetId,
        });
        if (result) {
          setSelectedAsset(result);
        }
        return result;
      } catch (error) {
        console.error("Asset selection failed:", error);
      }
    },
    [sendRequest],
  );

  const getAvailableAssets = useCallback(async (): Promise<
    Asset[] | undefined
  > => {
    try {
      const result = await sendRequest<Asset[]>("get_available_assets");
      const assets = Array.isArray(result) ? result : [];
      setAvailableAssets(assets);
      return assets;
    } catch (error) {
      console.error("Failed to get available assets:", error);
      setAvailableAssets([]); // Keep as empty array on error
    }
  }, [sendRequest]);

  // Places the current selected asset at the specific 3D coordinates
  const placeAssetAtPosition = useCallback(
    async (x: number, y: number, z: number): Promise<any> => {
      try {
        const result = await sendRequest("place_asset_at_position", {
          x,
          y,
          z,
        });
        if (result && selectedAsset) {
          const placedAsset: PlacedAsset = {
            id: Date.now(),
            asset: selectedAsset,
            position: { x, y, z },
            ...result,
          };
          setPlacedAssets((prev) => [...prev, placedAsset]);
        }
        return result;
      } catch (error) {
        console.error("Failed to place asset at position:", error);
        throw error;
      }
    },
    [sendRequest, selectedAsset],
  );

  // Toggle asset placement mode
  const toggleAssetPlacementMode = useCallback(async (): Promise<any> => {
    try {
      const result = await sendRequest("toggle_placement_mode");
      return result;
    } catch (error) {
      console.error("Failed to toggle placement mode:", error);
      throw error;
    }
  }, [sendRequest]);

  // Clear all placed assets
  const clearAllAssets = useCallback(async (): Promise<any> => {
    try {
      const result = await sendRequest("clear_all_assets");
      setPlacedAssets([]);
      return result;
    } catch (error) {
      console.error("Failed to clear all assets:", error);
      throw error;
    }
  }, [sendRequest]);

  return {
    // State
    fps,
    isConnected,
    availableAssets,
    selectedAsset,
    placedAssets,

    // Generic RPC methods
    sendRequest,
    sendNotification,
    onNotification,

    // Specific helpers
    selectTool,
    setRenderMode,
    getFps,
    selectAsset,
    getAvailableAssets,
    placeAssetAtPosition,
    toggleAssetPlacementMode,
    clearAllAssets,
  };
};
