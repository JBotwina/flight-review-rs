import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';
import AiWorkingState from '../AiWorkingState.svelte';

describe('AiWorkingState', () => {
	it('announces an active evidence-correlation pass', () => {
		render(AiWorkingState);

		const status = screen.getByRole('status');
		expect(status.getAttribute('aria-busy')).toBe('true');
		expect(status.textContent).toContain('Correlating flight evidence');
		expect(status.textContent).toContain('AI / WORKING');
	});
});
