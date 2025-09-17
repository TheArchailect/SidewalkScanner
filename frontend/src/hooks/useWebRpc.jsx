<<<<<<< HEAD
import { useState, useEffect, useCallback, useRef, use } from "react";
=======
"use client";

import { useState, useEffect, useCallback, useRef } from "react";
>>>>>>> feature_asset_library_native

export const useWebRpc = () => {
  const [fps, setFps] = useState(0);
  const [renderMode, setRenderMode] = useState("");
  const [isConnected, setIsConnected] = useState(false);
  const [availableAssets, setAvailableAssets] = useState([]);
  const [assetCategories, setAssetCategories] = useState([]);
  const [selectedAsset, setSelectedAsset] = useState([]);
  const [placedAssets, setPlacedAssets] = useState([]);
  const canvasRef = useRef(null);
  const requestIdCounter = useRef(1);
  const pendingRequests = useRef(new Map());
  const notificationHandlers = useRef(new Map());

  // Generate unique request ID
  const generateRequestId = useCallback(() => {
    return requestIdCounter.current++;
  }, []);

  // Send JSON-RPC request (expects response)
  const sendRequest = useCallback(
    (method, params = {}) => {
      return new Promise((resolve, reject) => {
        if (!canvasRef.current?.contentWindow) {
          reject(new Error("Canvas not ready"));
          return;
        }

        const id = generateRequestId();
        const request = {
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
    [generateRequestId],
  );

  // Send JSON-RPC notification (no response expected)
  const sendNotification = useCallback((method, params = {}) => {
    if (!canvasRef.current?.contentWindow) {
      return;
    }

    const notification = {
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
  }, []);

  // Register handler for incoming notifications
  const onNotification = useCallback((method, handler) => {
    notificationHandlers.current.set(method, handler);
  }, []);

  // Handle incoming messages from Bevy
  useEffect(() => {
    const handleMessage = (event) => {
      // Ensure message is from our iframe
      if (
        canvasRef.current &&
        event.source !== canvasRef.current.contentWindow
      ) {
        return;
      }

      try {
        const message = JSON.parse(event.data);

        if (message.method === "debug_message") {
          // console.log("[RUST DEBUG]", message.params.message);
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
            setFps(message.params.fps || 0);
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
  }, []);

  // Monitor iframe connection state
  useEffect(() => {
    const checkConnection = () => {
      if (canvasRef.current?.contentWindow) {
        setIsConnected(true);
        // Request initial FPS when connected
        sendRequest("get_fps").catch(() => {});
      } else {
        setIsConnected(false);
      }
    };

    const interval = setInterval(checkConnection, 1000);
    checkConnection();

    return () => clearInterval(interval);
  }, [sendRequest]);

  // Specific method helpers
  const selectTool = useCallback(
    async (tool) => {
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
    async (mode) => {
      try {
        // Send as notification since it's a setting change
        sendNotification("render_mode_changed", { mode });
        console.log(`Render mode set to: ${mode}`);
      } catch (error) {
        console.error("Render mode change failed:", error);
        throw error;
      }
    },
    [sendNotification],
  );

  const getFps = useCallback(async () => {
    try {
      const result = await sendRequest("get_fps");
      return result.fps;
    } catch (error) {
      return 0;
    }
  }, [sendRequest]);

  // Selects an asset by ID and updates local state
  const selectAsset = useCallback(
    async (assetId) => {
      try {
        const result = await sendRequest("select_asset", {asset_id: assetId})
        if (result) {
          setSelectedAsset(result)
        }
        return result
      } catch (error) {
        console.error("Asset selection failed:", error)
      }
    }, [sendRequest]);

  // Fetches all available assets from Bevy and updates state
  const getAvailableAssets = useCallback(async () => {
    try {
      const result = await sendRequest("get_available_assets")
      const assets = result || [] 
      setAvailableAssets(assets)
      return assets
    } catch (error) {
      console.error("Failed to get available assets:", error)
    }
  }, [sendRequest]);

  // Fetches categories 
  const getAssetCategories = useCallback(async () => {
    try {
      const result = await sendRequest("get_asset_category")
      const categories = result || []
      setAssetCategories(categories)
    } catch (error) {
      console.error("Failed to get asset categories:", error)
    }
  }, [sendRequest]);

  // Places the current selected asset at the specific 3D coordinates
  const placeAssetAtPosition = useCallback(
    async (x, y, z) => {
      try {
        const result = await sendRequest("place_asset_at_position", { x, y, z })
        if (result && selectedAsset) {    // Use the current selected asset
          const placedAsset = {
            id: Date.now(), // temporary ID
            asset: selectedAsset,         // selected asset
            position: { x, y, z },
            ...result,
          }
          setPlacedAssets((prev) => [...prev, placedAsset])
        }
        return result
      } catch (error) {
        console.error("Failed to place asset at position:", error)
        throw error
      }
    },
    [sendRequest, selectedAsset],
  )

  return {
    // State
    fps,
    //renderMode,
    isConnected,

    // Generic RPC methods
    sendRequest,
    sendNotification,
    onNotification,

    // Specific helpers
    selectTool,
    setRenderMode, // Added render mode method to exports
    getFps,

    selectAsset,
    getAvailableAssets,
    getAssetCategories,
    placeAssetAtPosition,

    // Canvas ref
    canvasRef,
  };
};
