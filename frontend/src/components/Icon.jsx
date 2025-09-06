import { useState, useEffect } from "react";

const Icon = ({
  name,
  size = 16,
  color = "currentColor",
  className = "",
  style = {},
}) => {
  const [svgContent, setSvgContent] = useState("");

  useEffect(() => {
    const loadSvg = async () => {
      try {
        const response = await fetch(`/icons/${name}.svg`);
        let text = await response.text();

        // Replace fill color in SVG content
        text = text.replace(/fill="[^"]*"/g, `fill="${color}"`);

        setSvgContent(text);
      } catch (error) {
        console.error(`Failed to load icon: ${name}`, error);
      }
    };

    loadSvg();
  }, [name, color]);

  if (!svgContent) {
    return <div style={{ width: size, height: size }} />;
  }

  return (
    <div
      className={className}
      style={{
        width: size,
        height: size,
        display: "inline-flex",
        alignItems: "center",
        justifyContent: "center",
        color: color,
        ...style,
      }}
      dangerouslySetInnerHTML={{ __html: svgContent }}
    />
  );
};

export default Icon;
