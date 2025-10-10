import type React from "react";
import { useState, useEffect, useRef, type RefObject } from "react";
import { useWebRpc } from "../hooks/useWebRpc";
import { theme, styleUtils } from "../theme";
import { ClassificationCategory } from "../hooks/useWebRpc";

interface PolygonToolPanelProps {
  isVisible: boolean;
  setHasSelection: React.Dispatch<React.SetStateAction<boolean>>;
  canvasRef: RefObject<HTMLIFrameElement | null>;
}

interface SelectableClass {
  classId: number;
  className: string;
  color: string;
  objectIds: number[];
  selected: boolean;
  selectedObjectIds: Set<number>;
}

type Operation = "hide" | "reclassify";

const PolygonToolPanel: React.FC<PolygonToolPanelProps> = ({
  isVisible,
  setHasSelection,
  canvasRef,
}) => {
  const {
    classificationCategories,
    getClassificationCategories,
    selectTool,
    clearTool,
    hidePointsInPolygon,
    reclassifyPointsInPolygon,
    setOnMouseEnterObjectID,
  } = useWebRpc(canvasRef);

  const [operation, setOperation] = useState<Operation>("hide");
  const [sourceClasses, setSourceClasses] = useState<SelectableClass[]>([]);
  const [targetClassId, setTargetClassId] = useState<number>(-1);

  const returnFocusToCanvas = (): void => {
    setTimeout(() => canvasRef.current?.focus(), 100);
  };

  const prettify = (id: string | undefined): string => {
    if (!id) return "Unknown";
    return id.replace(/[_-]+/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());
  };

  const colorForCategory = (catId: string): string => {
    if (catId === "vehicles") return theme.colors.primary.blue;
    if (catId === "vegetation") return theme.colors.success;
    if (catId === "infrastructure") return theme.colors.primary.orange;
    if (catId === "furniture") return "#8b5cf6";
    if (catId === "uncategorised") return theme.colors.gray[400];
    return theme.colors.primary.blue;
  };

  // Initialize source classes from classification categories
  useEffect(() => {
    if (!classificationCategories) return;
    console.log("classificationCategories: ", classificationCategories);
    const classes: SelectableClass[] = classificationCategories.map((cat) => ({
      classId: cat.class,
      className: prettify(cat.class_name),
      color: colorForCategory(cat.class.toString()),
      objectIds: cat.object_ids,
      selected: false,
      selectedObjectIds: new Set<number>(),
    }));

    setSourceClasses(
      classes.sort((a, b) => a.className.localeCompare(b.className)),
    );
    setTargetClassId(-1);
  }, [classificationCategories]);

  // Toggle class selection
  const toggleClass = (classId: number): void => {
    setSourceClasses((prev) =>
      prev.map((cls) =>
        cls.classId === classId
          ? { ...cls, selected: !cls.selected, selectedObjectIds: new Set() }
          : cls,
      ),
    );

    setHasSelection(true);
    returnFocusToCanvas();
  };

  // Toggle object ID selection within a class
  const toggleObjectId = (classId: number, objectId: number): void => {
    setSourceClasses((prev) =>
      prev.map((cls) => {
        if (cls.classId !== classId) return cls;

        const newSelectedIds = new Set(cls.selectedObjectIds);
        if (newSelectedIds.has(objectId)) {
          newSelectedIds.delete(objectId);
        } else {
          newSelectedIds.add(objectId);
        }

        return { ...cls, selectedObjectIds: newSelectedIds };
      }),
    );
    returnFocusToCanvas();
  };

  // Get selected classes with their optional object IDs
  const getSelectedPairs = (): Array<{
    class_id: number;
    object_id: number;
  }> => {
    return sourceClasses.flatMap((cls) => {
      if (!cls.selected) return [];

      // If specific object IDs are selected, use those
      if (cls.selectedObjectIds.size > 0) {
        return Array.from(cls.selectedObjectIds).map((objId) => ({
          class_id: cls.classId,
          object_id: objId,
        }));
      }

      // Otherwise, include all object IDs for this class
      return cls.objectIds.map((objId) => ({
        class_id: cls.classId,
        object_id: objId,
      }));
    });
  };

  const hasAnySelection = sourceClasses.some((cls) => cls.selected);

  const handleApply = async (): Promise<void> => {
    const selectedPairs = getSelectedPairs();

    if (operation === "hide") {
      await hidePointsInPolygon(selectedPairs);
    } else {
      console.log("reclassifyPointsInPolygon Component Before:");
      if (!targetClassId) return;

      console.log("reclassifyPointsInPolygon Component After:");
      // For reclassify, we need to pass the target class
      const targetClass = sourceClasses.find(
        (cls) => cls.classId === targetClassId,
      );

      let object_id_target =
        !targetClass || targetClass.objectIds.length === 0
          ? -1
          : targetClass.objectIds[0];

      await reclassifyPointsInPolygon(
        selectedPairs,
        targetClassId,
        object_id_target,
      );
    }

    returnFocusToCanvas();
  };

  useEffect(() => {
    if (!isVisible) return;
    selectTool("polygon").catch(console.error);
    getClassificationCategories();
  }, [isVisible]);

  const panelRef = useRef<HTMLDivElement | null>(null);
  useEffect(() => {
    if (isVisible) panelRef.current?.focus();
  }, [isVisible]);

  const handleCancel = (): void => {
    clearTool()
      .catch(console.error)
      .finally(() => setTimeout(() => canvasRef.current?.focus(), 0));
  };

  const handleKeyDownCapture = (
    e: React.KeyboardEvent<HTMLDivElement>,
  ): void => {
    if (e.key === "Escape" || e.key === "Esc") {
      e.preventDefault();
      e.stopPropagation();
      handleCancel();
    }
  };

  if (!isVisible) return null;

  const selectedClasses = sourceClasses.filter((cls) => cls.selected);

  return (
    <div
      ref={panelRef}
      tabIndex={-1}
      onKeyDownCapture={handleKeyDownCapture}
      style={{
        position: "fixed",
        right: theme.spacing[6],
        top: "70px",
        width: "360px",
        ...styleUtils.glassPanel("medium"),
        zIndex: theme.zIndex.modal,
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      {/* Header */}
      <div
        style={{
          padding: theme.spacing[5],
          borderBottom: `1px solid ${theme.colors.border.default}`,
        }}
      >
        <h3
          style={{
            margin: `0 0 ${theme.spacing[5]} 0`,
            ...styleUtils.text.subtitle(),
            textAlign: "center",
          }}
        >
          Polygon Tool
        </h3>

        <div
          style={{
            display: "flex",
            gap: theme.spacing[2],
            marginBottom: theme.spacing[4],
          }}
        >
          <button
            onClick={() => {
              setOperation("hide");
              returnFocusToCanvas();
            }}
            style={{
              ...styleUtils.buttonBase(),
              background:
                operation === "hide"
                  ? theme.colors.background.overlay
                  : theme.colors.background.card,
              border: `1px solid ${
                operation === "hide"
                  ? theme.colors.border.orangeStrong
                  : theme.colors.border.light
              }`,
              color:
                operation === "hide"
                  ? theme.colors.primary.orangeLight
                  : theme.colors.gray[500],
              padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
              borderRadius: theme.radius.md,
              fontSize: theme.fontSizes.sm,
              fontWeight: theme.fontWeights.medium,
              transition: theme.transitions.fast,
              flex: 1,
            }}
          >
            Hide Points
          </button>
          <button
            onClick={() => {
              setOperation("reclassify");
              returnFocusToCanvas();
            }}
            style={{
              ...styleUtils.buttonBase(),
              background:
                operation === "reclassify"
                  ? theme.colors.background.overlay
                  : theme.colors.background.card,
              border: `1px solid ${
                operation === "reclassify"
                  ? theme.colors.border.orangeStrong
                  : theme.colors.border.light
              }`,
              color:
                operation === "reclassify"
                  ? theme.colors.primary.orangeLight
                  : theme.colors.gray[500],
              padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
              borderRadius: theme.radius.md,
              fontSize: theme.fontSizes.sm,
              fontWeight: theme.fontWeights.medium,
              transition: theme.transitions.fast,
              flex: 1,
            }}
          >
            Reclassify Points
          </button>
        </div>

        <div
          style={{
            ...styleUtils.text.caption(),
            textAlign: "center",
            lineHeight: "1.4",
            background: theme.colors.background.card,
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            borderRadius: theme.radius.base,
            border: `1px solid ${theme.colors.border.light}`,
          }}
        >
          {operation === "hide"
            ? "Select class types to hide. Optionally refine by specific object IDs, or leave empty to affect all in polygon."
            : "Select class types to reclassify from, optionally refine by object IDs, then choose target class."}
        </div>
      </div>

      <div
        style={{
          padding: `${theme.spacing[4]} ${theme.spacing[5]}`,
          flex: 1,
          maxHeight: "400px",
          overflowY: "auto",
          display: "flex",
          flexDirection: "column",
          gap: theme.spacing[4],
        }}
      >
        {/* Source Selection */}
        <div
          style={{
            background: theme.colors.background.card,
            border: `1px solid ${theme.colors.border.default}`,
            borderRadius: theme.radius.md,
            padding: theme.spacing[4],
          }}
        >
          <h4
            style={{
              margin: `0 0 ${theme.spacing[4]} 0`,
              fontSize: theme.fontSizes.base,
              fontWeight: theme.fontWeights.semibold,
              color: theme.colors.white,
            }}
          >
            {operation === "hide" ? "What to Hide" : "What to Reclassify"}
          </h4>

          {/* Class Selection */}
          <div
            style={{
              display: "flex",
              flexWrap: "wrap",
              gap: theme.spacing[2],
              marginBottom: theme.spacing[4],
            }}
          >
            {sourceClasses.map((cls) => (
              <button
                key={cls.classId}
                onClick={() => toggleClass(cls.classId)}
                style={{
                  ...styleUtils.buttonBase(),
                  background: cls.selected
                    ? theme.colors.background.overlay
                    : theme.colors.background.card,
                  border: `1px solid ${
                    cls.selected ? cls.color : theme.colors.border.light
                  }`,
                  color: cls.selected ? cls.color : theme.colors.gray[400],
                  padding: `${theme.spacing[2]} ${theme.spacing[3]}`,
                  borderRadius: theme.radius.base,
                  fontSize: theme.fontSizes.sm,
                  fontWeight: cls.selected
                    ? theme.fontWeights.semibold
                    : theme.fontWeights.medium,
                  display: "flex",
                  alignItems: "center",
                  gap: theme.spacing[2],
                }}
              >
                <div
                  style={{
                    width: "8px",
                    height: "8px",
                    borderRadius: "50%",
                    background: cls.color,
                    opacity: cls.selected ? 1 : 0.5,
                  }}
                />
                {cls.className}
              </button>
            ))}
          </div>

          {/* Object ID refinement for selected classes */}
          {selectedClasses.length > 0 && (
            <div
              style={{
                borderTop: `1px solid ${theme.colors.border.light}`,
                paddingTop: theme.spacing[4],
              }}
            >
              <div
                style={{
                  fontSize: theme.fontSizes.xs,
                  color: theme.colors.gray[400],
                  marginBottom: theme.spacing[3],
                  fontWeight: theme.fontWeights.medium,
                }}
              >
                Optional: Refine by Object IDs
              </div>
              {selectedClasses.map((cls) => (
                <div
                  key={cls.classId}
                  style={{ marginBottom: theme.spacing[3] }}
                >
                  <div
                    style={{
                      fontSize: theme.fontSizes.xs,
                      color: cls.color,
                      marginBottom: theme.spacing[2],
                      fontWeight: theme.fontWeights.medium,
                    }}
                  >
                    {cls.className}
                  </div>
                  <div
                    style={{
                      display: "flex",
                      flexWrap: "wrap",
                      gap: theme.spacing[1],
                      paddingLeft: theme.spacing[3],
                    }}
                  >
                    {cls.objectIds.map((objId) => (
                      <button
                        key={`${cls.classId}-${objId}`}
                        onClick={() => toggleObjectId(cls.classId, objId)}
                        onMouseEnter={() => {
                          setOnMouseEnterObjectID(objId);
                        }}
                        onMouseLeave={() => {
                          setOnMouseEnterObjectID(-1);
                        }}
                        style={{
                          ...styleUtils.toolItem(
                            cls.selectedObjectIds.has(objId),
                          ),
                          padding: `${theme.spacing[1]} ${theme.spacing[2]}`,
                          fontSize: theme.fontSizes.xs,
                        }}
                      >
                        {objId}
                      </button>
                    ))}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        {/* Target Selection (reclassify only) */}
        {operation === "reclassify" && (
          <div
            style={{
              background: theme.colors.background.overlay,
              border: `1px solid ${theme.colors.border.orangeStrong}`,
              borderRadius: theme.radius.md,
              padding: theme.spacing[4],
            }}
          >
            <h4
              style={{
                margin: `0 0 ${theme.spacing[3]} 0`,
                fontSize: theme.fontSizes.base,
                fontWeight: theme.fontWeights.semibold,
                color: theme.colors.primary.orangeLight,
                display: "flex",
                alignItems: "center",
                gap: theme.spacing[2],
              }}
            >
              <span>→</span>
              Reclassify To
            </h4>

            <select
              value={targetClassId}
              onChange={(e) => {
                setTargetClassId(Number(e.target.value));
                returnFocusToCanvas();
              }}
              style={{
                ...styleUtils.inputField(),
                padding: `${theme.spacing[3]} ${theme.spacing[3]}`,
                fontSize: theme.fontSizes.sm,
                cursor: "pointer",
              }}
            >
              <option
                value=""
                style={{
                  background: theme.colors.gray[800],
                  color: theme.colors.white,
                }}
              >
                Choose target class...
              </option>
              {sourceClasses.map((cls) => (
                <option
                  key={cls.classId}
                  value={cls.classId}
                  style={{
                    background: theme.colors.gray[800],
                    color: theme.colors.white,
                  }}
                >
                  {cls.className}
                </option>
              ))}
            </select>
          </div>
        )}
      </div>

      {/* Footer Actions */}
      <div
        style={{
          padding: `${theme.spacing[4]} ${theme.spacing[5]}`,
          borderTop: `1px solid ${theme.colors.border.default}`,
          display: "flex",
          gap: theme.spacing[3],
        }}
      >
        {/*<button
          onClick={handleCancel}
          style={{
            ...styleUtils.buttonGhost(),
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            fontSize: theme.fontSizes.sm,
            fontWeight: theme.fontWeights.semibold,
            flex: "1",
          }}
        >
          Cancel
        </button>*/}
        <button
          onClick={handleApply}
          disabled={operation === "reclassify" && !targetClassId}
          style={{
            ...styleUtils.buttonBase(),
            background:
              operation === "reclassify" && !targetClassId
                ? theme.colors.background.card
                : theme.colors.background.overlay,
            border: `1px solid ${
              operation === "reclassify" && !targetClassId
                ? theme.colors.border.light
                : theme.colors.border.orangeStrong
            }`,
            color:
              operation === "reclassify" && !targetClassId
                ? theme.colors.gray[600]
                : theme.colors.primary.orangeLight,
            padding: `${theme.spacing[3]} ${theme.spacing[4]}`,
            fontSize: theme.fontSizes.sm,
            fontWeight: theme.fontWeights.semibold,
            cursor:
              operation === "reclassify" && !targetClassId
                ? "not-allowed"
                : "pointer",
            transition: theme.transitions.fast,
            flex: "2",
          }}
        >
          {operation === "hide"
            ? hasAnySelection
              ? "Hide Selected"
              : "Hide All"
            : hasAnySelection
              ? "Reclassify Selected"
              : "Reclassify All"}
        </button>
      </div>
    </div>
  );
};

export default PolygonToolPanel;
