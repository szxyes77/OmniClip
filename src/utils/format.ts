import { format } from "date-fns";
import { zhCN } from "date-fns/locale";

export function formatRelativeTime(dateString: string): string {
  try {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMinutes = diffMs / (1000 * 60);
    const diffHours = diffMs / (1000 * 60 * 60);
    const diffDays = diffMs / (1000 * 60 * 60 * 24);

    if (diffMinutes < 1) return "Just now";
    if (diffMinutes < 60) return `${Math.floor(diffMinutes)} min ago`;
    if (diffHours < 24) return `${Math.floor(diffHours)} hours ago`;
    if (diffDays < 7) return `${Math.floor(diffDays)} days ago`;
    return format(date, "MMM d, yyyy", { locale: zhCN });
  } catch {
    return dateString;
  }
}

export function formatFullDate(dateString: string): string {
  try {
    const date = new Date(dateString);
    return format(date, "yyyy-MM-dd HH:mm:ss", { locale: zhCN });
  } catch {
    return dateString;
  }
}

export function truncateText(text: string, maxLength: number = 100): string {
  if (text.length <= maxLength) return text;
  return text.slice(0, maxLength) + "...";
}
