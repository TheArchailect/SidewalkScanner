import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import ScannerApp from "./pages/ScannerApp.jsx";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <ScannerApp />
  </StrictMode>,
);
