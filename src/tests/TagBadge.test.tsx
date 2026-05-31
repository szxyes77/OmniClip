import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { TagBadge } from '../components/main/TagBadge';

describe('TagBadge', () => {
  it('renders tag name correctly', () => {
    render(<TagBadge name="Work" color="#60A5FA" />);
    expect(screen.getByText('Work')).toBeInTheDocument();
  });

  it('applies correct color to the badge', () => {
    render(<TagBadge name="Important" color="#F472B6" />);
    const badge = screen.getByText('Important');
    expect(badge).toHaveStyle({ color: '#F472B6' });
  });

  it('renders Chinese characters correctly', () => {
    render(<TagBadge name="测试标签" color="#34D399" />);
    expect(screen.getByText('测试标签')).toBeInTheDocument();
  });

  it('renders with soft color palette', () => {
    const softColors = ['#F472B6', '#A78BFA', '#60A5FA', '#34D399', '#FBBF24'];
    softColors.forEach((color) => {
      const { unmount } = render(<TagBadge name="Test" color={color} />);
      const badge = screen.getByText('Test');
      expect(badge).toHaveStyle({ color });
      unmount();
    });
  });
});
