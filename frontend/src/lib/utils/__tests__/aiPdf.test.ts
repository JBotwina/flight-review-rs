import { describe, expect, it, vi } from 'vitest';
import type { AiAnalysis } from '$lib/types';
import { aiAnalysisPdfFilename, buildAiAnalysisTypst } from '../aiPdf';

const analysis: AiAnalysis = {
  schema_version: 1,
  generated_at: '2026-07-10T14:22:00Z',
  requested_model: 'provider/requested',
  model: 'provider/model-v1',
  summary: 'The pilot said "check \\ logs" - then landed safely.',
  risk_level: 'moderate',
  confidence: 0.876,
  findings: [
    {
      category: 'power',
      severity: 'warning',
      title: 'Voltage sag',
      explanation: 'Battery voltage dropped during climb.',
      evidence: ['12.4 V at T+10.2s', 'No brownout was detected.'],
      time_range_s: { start: 10.2, end: 12.8 },
    },
  ],
  positive_observations: ['Estimator remained healthy.'],
  recommendations: [
    {
      priority: 'high',
      action: 'Inspect the battery.',
      rationale: 'Rule out excessive internal resistance.',
    },
  ],
  limitations: ['Only logged evidence was reviewed.'],
  usage: { prompt_tokens: 100, completion_tokens: 50, total_tokens: 150, cost: 0.01 },
};

describe('buildAiAnalysisTypst', () => {
  it('includes every analysis section and report metadata', () => {
    const source = buildAiAnalysisTypst(analysis, 'log/abc 123');

    expect(source).toContain('00 / EXECUTIVE READOUT');
    expect(source).toContain('01 / OBSERVATIONS');
    expect(source).toContain('02 / NOMINAL SIGNALS');
    expect(source).toContain('03 / NEXT ACTIONS');
    expect(source).toContain('04 / ANALYSIS ENVELOPE');
    expect(source).toContain('log/abc 123');
    expect(source).toContain('provider/model-v1');
    expect(source).toContain('#let confidence = 88');
    expect(source).toContain('#let token-load = 150');
  });

  it('escapes model text and normalizes typographic dashes for PDF portability', () => {
    const source = buildAiAnalysisTypst({
      ...analysis,
      summary: 'Quoted "value" with \\ path and en dash \u2013 em dash \u2014.',
    });

    expect(source).toContain('Quoted \\"value\\" with \\\\ path and en dash - em dash -.');
    expect(source).not.toContain('\u2013');
    expect(source).not.toContain('\u2014');
  });

  it('emits valid empty Typst arrays for analyses without list content', () => {
    const source = buildAiAnalysisTypst({
      ...analysis,
      confidence: null,
      findings: [],
      positive_observations: [],
      recommendations: [],
      limitations: [],
      usage: null,
    });

    expect(source).toContain('#let confidence = none');
    expect(source).toContain('#let findings = ()');
    expect(source).toContain('#let positives = ()');
    expect(source).toContain('#let recommendations = ()');
    expect(source).toContain('#let limitations = ()');
    expect(source).toContain('#let token-load = none');
  });
});

describe('aiAnalysisPdfFilename', () => {
  it('creates a stable, filesystem-safe filename', () => {
    expect(aiAnalysisPdfFilename('log / PX4 #42', analysis.generated_at))
      .toBe('flight-review-log-PX4-42-ai-analysis-2026-07-10.pdf');
  });

  it('falls back to the current date for an invalid generated timestamp', () => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2026-08-04T12:00:00Z'));
    expect(aiAnalysisPdfFilename(undefined, 'invalid'))
      .toBe('flight-review-flight-ai-analysis-2026-08-04.pdf');
    vi.useRealTimers();
  });
});
