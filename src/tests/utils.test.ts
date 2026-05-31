import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { formatRelativeTime } from '../utils/format';
import { useDebounce } from '../hooks/useDebounce';
import { renderHook, act } from '@testing-library/react';

describe('formatRelativeTime', () => {
  const now = new Date();

  it('formats "just now" for recent timestamps', () => {
    const recent = new Date(now.getTime() - 10000).toISOString();
    expect(formatRelativeTime(recent)).toBe('Just now');
  });

  it('formats minutes ago', () => {
    const fiveMinAgo = new Date(now.getTime() - 5 * 60 * 1000).toISOString();
    expect(formatRelativeTime(fiveMinAgo)).toContain('m');
  });

  it('formats hours ago', () => {
    const threeHoursAgo = new Date(now.getTime() - 3 * 60 * 60 * 1000).toISOString();
    expect(formatRelativeTime(threeHoursAgo)).toContain('h');
  });

  it('formats days ago', () => {
    const twoDaysAgo = new Date(now.getTime() - 2 * 24 * 60 * 60 * 1000).toISOString();
    expect(formatRelativeTime(twoDaysAgo)).toContain('d');
  });
});

describe('useDebounce', () => {
  beforeEach(() => {
    vi.useFakeTimers();
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns initial value immediately', () => {
    const { result } = renderHook(() => useDebounce('test', 300));
    expect(result.current).toBe('test');
  });

  it('debounces value changes', () => {
    const { result, rerender } = renderHook(
      ({ value }) => useDebounce(value, 300),
      { initialProps: { value: 'initial' } }
    );

    expect(result.current).toBe('initial');

    rerender({ value: 'updated' });
    expect(result.current).toBe('initial');

    act(() => {
      vi.advanceTimersByTime(300);
    });
    expect(result.current).toBe('updated');
  });

  it('cancels previous timer on rapid updates', () => {
    const { result, rerender } = renderHook(
      ({ value }) => useDebounce(value, 300),
      { initialProps: { value: 'first' } }
    );

    rerender({ value: 'second' });
    act(() => {
      vi.advanceTimersByTime(150);
    });

    rerender({ value: 'third' });
    act(() => {
      vi.advanceTimersByTime(150);
    });
    expect(result.current).toBe('first');

    act(() => {
      vi.advanceTimersByTime(150);
    });
    expect(result.current).toBe('third');
  });
});
