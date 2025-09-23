import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import ScannerApp from "./pages/ScannerApp.jsx";

createRoot(document.getElementById("root")).render(
  <StrictMode>
    <ScannerApp />
  </StrictMode>,
);

document.addEventListener("contextmenu", function (e) {
  e.preventDefault();
  console.log("Right-click disabled. Show custom menu.");
});
