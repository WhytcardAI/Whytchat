export default function ModeButton({ mode, currentMode, label, onClick }) {
  return (
    <button
      onClick={onClick}
      className={`px-3 py-1 rounded-md text-sm font-medium transition-colors border ${
        mode === currentMode
          ? "bg-primary text-primary-foreground border-primary"
          : "bg-secondary text-foreground border-border hover:bg-secondary/80"
      }`}
      type="button"
    >
      {label}
    </button>
  );
}
