import { useState, useRef, useEffect, useId } from "react";

export interface SelectOption {
  value: string;
  label: string;
}

interface Props {
  id?: string;
  options: SelectOption[];
  value: string;
  onChange: (value: string) => void;
  disabled?: boolean;
}

export function ThemedSelect({ id, options, value, onChange, disabled = false }: Props) {
  const [isOpen, setIsOpen] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);
  const generatedId = useId();
  const buttonId = id ?? generatedId;

  const selectedLabel = options.find((o) => o.value === value)?.label ?? value;

  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setIsOpen(false);
      }
    }
    if (isOpen) document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [isOpen]);

  function handleKeyDown(e: React.KeyboardEvent) {
    if (e.key === "Escape") { setIsOpen(false); return; }
    if (e.key === "Enter" || e.key === " ") { e.preventDefault(); setIsOpen((o) => !o); return; }
    if (!isOpen) return;
    const currentIndex = options.findIndex((o) => o.value === value);
    if (e.key === "ArrowDown") {
      e.preventDefault();
      const next = options[Math.min(currentIndex + 1, options.length - 1)];
      if (next) onChange(next.value);
    }
    if (e.key === "ArrowUp") {
      e.preventDefault();
      const prev = options[Math.max(currentIndex - 1, 0)];
      if (prev) onChange(prev.value);
    }
  }

  return (
    <div ref={containerRef} style={{ position: "relative" }}>
      <button
        id={buttonId}
        type="button"
        disabled={disabled}
        onClick={() => !disabled && setIsOpen((o) => !o)}
        onKeyDown={handleKeyDown}
        aria-haspopup="listbox"
        aria-expanded={isOpen}
        style={{
          width: "100%",
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "0.5rem 0.75rem",
          backgroundColor: "var(--color-bg)",
          border: "1px solid var(--color-border)",
          borderRadius: "var(--border-radius)",
          color: disabled ? "var(--color-text-muted)" : "var(--color-text)",
          fontFamily: "var(--font-body)",
          fontSize: "0.875rem",
          cursor: disabled ? "not-allowed" : "pointer",
          textAlign: "left",
          opacity: disabled ? 0.6 : 1,
        }}
      >
        <span style={{ flex: 1, overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>
          {selectedLabel}
        </span>
        <span
          style={{
            marginLeft: "0.5rem",
            fontSize: "0.7rem",
            color: "var(--color-text-muted)",
            transition: "transform 0.15s ease",
            transform: isOpen ? "rotate(180deg)" : "rotate(0deg)",
            flexShrink: 0,
          }}
        >
          ▼
        </span>
      </button>

      {isOpen && (
        <ul
          role="listbox"
          style={{
            position: "absolute",
            top: "calc(100% + 4px)",
            left: 0,
            right: 0,
            zIndex: 9999,
            backgroundColor: "var(--color-surface)",
            border: "1px solid var(--color-border)",
            borderRadius: "var(--border-radius)",
            padding: "0.25rem 0",
            margin: 0,
            listStyle: "none",
            maxHeight: "16rem",
            overflowY: "auto",
            boxShadow: "0 8px 24px rgba(0,0,0,0.5)",
          }}
        >
          {options.map((option) => {
            const isSelected = option.value === value;
            return (
              <li
                key={option.value}
                role="option"
                aria-selected={isSelected}
                onClick={() => { onChange(option.value); setIsOpen(false); }}
                style={{
                  padding: "0.5rem 0.75rem",
                  fontSize: "0.875rem",
                  fontFamily: "var(--font-body)",
                  cursor: "pointer",
                  color: isSelected ? "var(--color-primary)" : "var(--color-text)",
                  backgroundColor: isSelected ? "var(--color-border)" : "transparent",
                  display: "flex",
                  alignItems: "center",
                  justifyContent: "space-between",
                }}
                onMouseEnter={(e) => {
                  if (!isSelected) (e.currentTarget as HTMLElement).style.backgroundColor = "var(--color-border)";
                }}
                onMouseLeave={(e) => {
                  if (!isSelected) (e.currentTarget as HTMLElement).style.backgroundColor = "transparent";
                }}
              >
                <span>{option.label}</span>
                {isSelected && (
                  <span style={{ fontSize: "0.7rem", color: "var(--color-primary)" }}>✓</span>
                )}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}
