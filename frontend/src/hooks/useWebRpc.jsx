"use client";

import { useState, useEffect, useCallback, useRef } from "react";

export const useWebRpc = () => {
  const [fps, setFps] = useState(0);
  const [isConnected, setIsConnected] = useState(false);
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

  return {
    // State
    fps,
    isConnected,

    // Generic RPC methods
    sendRequest,
    sendNotification,
    onNotification,

    // Specific helpers
    selectTool,
    setRenderMode, // Added render mode method to exports
    getFps,

    // Canvas ref
    canvasRef,
  };
};
