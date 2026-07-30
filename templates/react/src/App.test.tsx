import { describe, expect, it } from 'vitest';
import { render, screen } from '@testing-library/react';
import App from './App';

describe('App', () => {
  it('renders the welcome heading', () => {
    render(<App />);
    expect(screen.getByText(/Welcome to/i)).toBeInTheDocument();
  });
});
