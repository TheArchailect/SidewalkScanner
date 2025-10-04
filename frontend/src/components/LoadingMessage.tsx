import type React from "react";
import { theme, styleUtils, createStyles } from "../theme";

type Props = {
  allFileLoadProgress: Record<string, number>;
};

const LoadingPanel: React.FC<Props> = ({ allFileLoadProgress }) => {
  const styles = createStyles({
    overlay: {
      position: "fixed",
      top: 0,
      left: 0,
      right: 0,
      bottom: 0,
      backgroundColor: theme.colors.background.overlay,
      backdropFilter: theme.backdropBlur.base,
      display: "flex",
      justifyContent: "center",
      alignItems: "center",
      zIndex: theme.zIndex.overlay,
    },
    box: {
      ...styleUtils.glassPanel("light"),
      padding: theme.spacing[8],
      minWidth: "350px",
      textAlign: "left",
      color: theme.colors.white,
    },
    title: {
      ...styleUtils.text.title(),
      fontSize: theme.fontSizes["3xl"],
      margin: `0 0 ${theme.spacing[2]} 0`,
    },
    subtitle: {
      ...styleUtils.text.body(),
      color: theme.colors.gray[200],
      fontSize: theme.fontSizes.md,
      margin: `0 0 ${theme.spacing[6]} 0`,
      lineHeight: "1.4",
    },
    progressItem: {
      display: "flex",
      justifyContent: "space-between",
      alignItems: "center",
      marginBottom: theme.spacing[3],
      padding: theme.spacing[2],
      background: theme.colors.background.input,
      borderRadius: theme.radius.md,
      border: `1px solid ${theme.colors.border.default}`,
    },
    fileName: {
      ...styleUtils.text.body(),
      fontSize: theme.fontSizes.md,
    },
    loading: {
      ...styleUtils.text.body(),
      color: theme.colors.primary.orange,
      fontSize: theme.fontSizes.md,
      fontStyle: "italic",
    },
    done: {
      ...styleUtils.text.body(),
      color: theme.colors.primary.blue,
      fontSize: theme.fontSizes.md,
      fontWeight: theme.fontWeights.medium,
    },
  });

  return (
    <div style={styles.overlay}>
      <div style={styles.box}>
        <h2 style={styles.title}>Loading data</h2>
        <p style={styles.subtitle}>
          Depending on your connection, this may take multiple minutes
        </p>
        {Object.entries(allFileLoadProgress).map(([key, value]) => (
          <div key={key} style={styles.progressItem}>
            <strong style={styles.fileName}>{key}</strong>:
            <span style={value === 0 ? styles.loading : styles.done}>
              {value === 0 ? "loading..." : "done"}
            </span>
          </div>
        ))}
      </div>
    </div>
  );
};

export default LoadingPanel;
