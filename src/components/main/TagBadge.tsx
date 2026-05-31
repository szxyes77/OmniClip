export function TagBadge({ name, color }: { name: string; color: string }) {
  return (
    <span
      className="inline-flex items-center px-1.5 py-0.5 rounded-full text-xs font-medium"
      style={{
        backgroundColor: `${color}20`,
        color: color,
        border: `1px solid ${color}30`,
      }}
    >
      {name}
    </span>
  );
}
