export const theme = {
  // Typography
  fonts: {
    primary:
      '"Roboto", -apple-system, BlinkMacSystemFont, "Segoe UI", system-ui, sans-serif',
    mono: '"Roboto Mono", "SF Mono", Monaco, Inconsolata, "Fira Code", "Droid Sans Mono", "Courier New", monospace',
  },

  fontSizes: {
    xs: "10px",
    sm: "11px",
    base: "12px",
    md: "13px",
    lg: "14px",
    xl: "16px",
    "2xl": "18px",
    "3xl": "24px",
  },

  fontWeights: {
    normal: "400",
    medium: "500",
    semibold: "600",
    bold: "700",
  },

  // Color Palette
  colors: {
    // Primary colors from your app
    primary: {
      orange: "#ff9700",
      orangeLight: "#ffb366",
      orangeDark: "#e6880a",
      blue: "#0068ff",
      blueLight: "#66a3ff",
      blueDark: "#0056d3",
    },

    // Semantic colors
    success: "#22c55e",
    warning: "#f59e0b",
    error: "#ef4444",
    info: "#3b82f6",

    // Neutral grays
    white: "#ffffff",
    gray: {
      50: "rgba(255, 255, 255, 0.95)",
      100: "rgba(255, 255, 255, 0.9)",
      200: "rgba(255, 255, 255, 0.8)",
      300: "rgba(255, 255, 255, 0.6)",
      400: "rgba(255, 255, 255, 0.4)",
      500: "#999999",
      600: "#666666",
      700: "#333333",
      800: "#1a1a1a",
      900: "#000000",
    },

    // Background variations (glassmorphism)
    background: {
      overlay: "rgba(0, 0, 0, 0.6)",
      modal: "rgba(0, 0, 0, 0.9)",
      panel: "rgba(0, 0, 0, 0.4)",
      card: "rgba(0, 0, 0, 0.3)",
      input: "rgba(255, 255, 255, 0.05)",
      hover: "rgba(255, 255, 255, 0.1)",
    },

    // Border variations
    border: {
      default: "rgba(255, 255, 255, 0.08)",
      light: "rgba(255, 255, 255, 0.1)",
      medium: "rgba(255, 255, 255, 0.2)",
      orange: "rgba(255, 151, 0, 0.3)",
      orangeStrong: "rgba(255, 151, 0, 0.8)",
      blue: "rgba(0, 104, 255, 0.3)",
      blueStrong: "rgba(0, 104, 255, 0.8)",
    },
  },

  // Spacing scale
  spacing: {
    0: "0px",
    1: "4px",
    2: "6px",
    3: "8px",
    4: "12px",
    5: "16px",
    6: "20px",
    7: "24px",
    8: "32px",
    9: "40px",
    10: "48px",
  },

  // Border radius
  radius: {
    none: "0px",
    sm: "3px",
    base: "4px",
    md: "6px",
    lg: "8px",
    xl: "12px",
  },

  // Common shadows
  shadows: {
    sm: "0 1px 2px 0 rgba(0, 0, 0, 0.05)",
    base: "0 1px 3px 0 rgba(0, 0, 0, 0.1), 0 1px 2px 0 rgba(0, 0, 0, 0.06)",
    lg: "0 10px 15px -3px rgba(0, 0, 0, 0.1), 0 4px 6px -2px rgba(0, 0, 0, 0.05)",
  },

  // Backdrop filters
  backdropBlur: {
    sm: "blur(4px)",
    base: "blur(20px)",
    lg: "blur(22px)",
  },

  // Transitions
  transitions: {
    fast: "all 0.15s ease",
    base: "all 0.2s ease",
    slow: "all 0.3s ease",
  },

  // Z-index scale
  zIndex: {
    dropdown: 10,
    modal: 15,
    overlay: 9999,
    highlight: 10000,
  },
} as const;

// Utility functions for common style patterns
export const styleUtils = {
  // Glassmorphism panel
  glassPanel: (variant: "light" | "medium" | "dark" = "medium") => ({
    background:
      variant === "light"
        ? theme.colors.background.input
        : variant === "medium"
          ? theme.colors.background.panel
          : theme.colors.background.modal,
    backdropFilter: theme.backdropBlur.base,
    border: `1px solid ${theme.colors.border.default}`,
    borderRadius: theme.radius.lg,
  }),

  // Button base styles
  buttonBase: () => ({
    display: "inline-flex" as const,
    alignItems: "center" as const,
    justifyContent: "center" as const,
    fontFamily: theme.fonts.primary,
    fontWeight: theme.fontWeights.medium,
    borderRadius: theme.radius.md,
    border: "none" as const,
    outline: "none" as const,
    cursor: "pointer" as const,
    transition: theme.transitions.base,
    userSelect: "none" as const,
  }),

  // Primary button
  buttonPrimary: () => ({
    ...styleUtils.buttonBase(),
    background: theme.colors.primary.blue,
    color: theme.colors.white,
    "&:hover": {
      background: theme.colors.primary.blueDark,
    },
  }),

  // Orange accent button
  buttonOrange: () => ({
    ...styleUtils.buttonBase(),
    background: `rgba(255, 151, 0, 0.2)`,
    border: `1px solid ${theme.colors.border.orange}`,
    color: theme.colors.primary.orange,
    "&:hover": {
      background: `rgba(255, 151, 0, 0.3)`,
    },
  }),

  // Ghost button
  buttonGhost: () => ({
    ...styleUtils.buttonBase(),
    background: "transparent",
    border: `1px solid ${theme.colors.border.light}`,
    color: theme.colors.gray[200],
    "&:hover": {
      background: theme.colors.background.hover,
    },
  }),

  // Input field
  inputField: () => ({
    background: theme.colors.background.input,
    border: `1px solid ${theme.colors.border.default}`,
    borderRadius: theme.radius.base,
    color: theme.colors.white,
    fontFamily: theme.fonts.primary,
    fontSize: theme.fontSizes.base,
    outline: "none" as const,
    transition: theme.transitions.base,
    "&:focus": {
      borderColor: theme.colors.border.light,
    },
  }),

  // Tool item (for tooltips, panels, etc.)
  toolItem: (isActive = false) => ({
    background: isActive
      ? `rgba(255, 151, 0, 0.2)`
      : theme.colors.background.input,
    border: `1px solid ${isActive ? theme.colors.border.orange : theme.colors.border.default}`,
    borderRadius: theme.radius.md,
    color: isActive ? theme.colors.primary.orange : theme.colors.gray[200],
    cursor: "pointer" as const,
    transition: theme.transitions.base,
  }),

  // Typography helpers
  text: {
    title: () => ({
      fontFamily: theme.fonts.primary,
      fontSize: theme.fontSizes.xl,
      fontWeight: theme.fontWeights.bold,
      color: theme.colors.primary.orange,
    }),

    subtitle: () => ({
      fontFamily: theme.fonts.primary,
      fontSize: theme.fontSizes.lg,
      fontWeight: theme.fontWeights.semibold,
      color: theme.colors.primary.orange,
    }),

    body: () => ({
      fontFamily: theme.fonts.primary,
      fontSize: theme.fontSizes.base,
      fontWeight: theme.fontWeights.normal,
      color: theme.colors.gray[200],
    }),

    caption: () => ({
      fontFamily: theme.fonts.primary,
      fontSize: theme.fontSizes.sm,
      fontWeight: theme.fontWeights.normal,
      color: theme.colors.gray[500],
    }),

    mono: () => ({
      fontFamily: theme.fonts.mono,
      fontSize: theme.fontSizes.sm,
      color: theme.colors.primary.orange,
    }),
  },
};

// CSS-in-JS helper for inline styles (React compatible)
export const createStyles = <T extends Record<string, React.CSSProperties>>(
  styles: T,
): T => styles;

// Global CSS that should be added to your app
export const globalCSS = `
  @import url('https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;600;700&family=Roboto+Mono:wght@400;500;600&display=swap');

  * {
    box-sizing: border-box;
  }

  body {
    font-family: ${theme.fonts.primary};
    margin: 0;
    padding: 0;
    background: ${theme.colors.gray[900]};
    color: ${theme.colors.white};
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
  }

  button {
    font-family: ${theme.fonts.primary};
  }

  input, textarea, select {
    font-family: ${theme.fonts.primary};
  }
`;

export default theme;
