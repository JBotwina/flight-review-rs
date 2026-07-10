import type { AiAnalysis, AiFindingSeverity, AiRiskLevel } from '$lib/types';

const PDF_MIME_TYPE = 'application/pdf';

type TypstModule = typeof import('@myriaddreamin/typst.ts');

let typstPromise: Promise<TypstModule['$typst']> | null = null;

function cleanText(value: string): string {
  return value
    .replace(/[\u2010-\u2015\u2212]/g, '-')
    .replace(/\u00a0/g, ' ')
    .replace(/[\u0000-\u0008\u000b\u000c\u000e-\u001f\u007f]/g, '');
}

function typstString(value: string): string {
  return JSON.stringify(cleanText(value));
}

function typstArray(values: string[]): string {
  if (values.length === 0) return '()';
  return `(${values.map((value) => `${typstString(value)},`).join('')})`;
}

function riskLabel(risk: AiRiskLevel): string {
  return risk === 'unknown' ? 'UNRATED' : `${risk.toUpperCase()} RISK`;
}

function severityCode(severity: AiFindingSeverity): string {
  switch (severity) {
    case 'critical': return 'CRT';
    case 'warning': return 'WRN';
    case 'info': return 'INF';
    default: return 'UNK';
  }
}

function timeLabel(start: number, end: number | null): string {
  return end == null ? `T+${start.toFixed(1)}s` : `T+${start.toFixed(1)}-${end.toFixed(1)}s`;
}

function safeIsoDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : date.toISOString().replace('T', ' ').replace('.000Z', 'Z');
}

function optionalNumber(value: number | null | undefined): string {
  return value == null ? 'none' : String(value);
}

function findingData(analysis: AiAnalysis): string {
  if (analysis.findings.length === 0) return '()';
  return `(${analysis.findings.map((finding) => `(
    severity: ${typstString(finding.severity)},
    code: ${typstString(severityCode(finding.severity))},
    category: ${typstString(finding.category)},
    title: ${typstString(finding.title)},
    explanation: ${typstString(finding.explanation)},
    evidence: ${typstArray(finding.evidence)},
    time: ${finding.time_range_s ? typstString(timeLabel(finding.time_range_s.start, finding.time_range_s.end)) : 'none'},
  ),`).join('')})`;
}

function recommendationData(analysis: AiAnalysis): string {
  if (analysis.recommendations.length === 0) return '()';
  return `(${analysis.recommendations.map((recommendation) => `(
    priority: ${typstString(recommendation.priority.toUpperCase())},
    action: ${typstString(recommendation.action)},
    rationale: ${typstString(recommendation.rationale)},
  ),`).join('')})`;
}

export function buildAiAnalysisTypst(analysis: AiAnalysis, reportId?: string): string {
  const generatedAt = safeIsoDate(analysis.generated_at);
  const confidencePercent = analysis.confidence == null ? null : Math.round(analysis.confidence * 100);
  const reportReference = cleanText(reportId?.trim() || 'SAVED ANALYSIS');

  return `#let ink = rgb("#10262d")
#let body-ink = rgb("#1a3339")
#let navy = rgb("#071823")
#let teal = rgb("#29777d")
#let acid = rgb("#c8ef4b")
#let muted = rgb("#5a6e71")
#let line-color = rgb("#cad3cf")
#let paper = rgb("#f8f8f3")
#let soft = rgb("#edf1ed")
#let warning = rgb("#a66b08")
#let critical = rgb("#b43d34")
#let mono = "DejaVu Sans Mono"
#let prose = "DejaVu Sans"

#let report-id = ${typstString(reportReference)}
#let generated-at = ${typstString(generatedAt)}
#let model = ${typstString(analysis.model)}
#let schema-version = ${analysis.schema_version}
#let summary = ${typstString(analysis.summary)}
#let risk = ${typstString(analysis.risk_level)}
#let risk-label = ${typstString(riskLabel(analysis.risk_level))}
#let confidence = ${optionalNumber(confidencePercent)}
#let findings = ${findingData(analysis)}
#let positives = ${typstArray(analysis.positive_observations)}
#let recommendations = ${recommendationData(analysis)}
#let limitations = ${typstArray(analysis.limitations)}
#let token-load = ${optionalNumber(analysis.usage?.total_tokens)}

#set document(
  title: "Flight Intelligence - AI Analysis",
  author: "Flight Review",
  keywords: ("PX4", "flight analysis", "engineering brief"),
)
#set page(
  paper: "a4",
  fill: paper,
  margin: (x: 16mm, top: 15mm, bottom: 19mm),
  footer: context {
    set text(font: mono, size: 6.5pt, fill: muted, tracking: 0pt)
    line(length: 100%, stroke: 0.5pt + line-color)
    v(4pt)
    grid(
      columns: (1fr, auto),
      [FLIGHT REVIEW / ENGINEERING AID],
      [PAGE #counter(page).display()],
    )
  },
)
#set text(font: prose, size: 9pt, fill: ink, tracking: 0pt)
#set par(justify: false, leading: 0.72em)

#let eyebrow(body) = text(font: mono, size: 6.5pt, weight: "bold", tracking: 0.2pt, fill: teal, body)
#let section-title(index, title) = block(above: 8pt, below: 7pt)[
  #eyebrow(index)
  #v(2pt)
  #text(size: 15pt, weight: "bold")[#title]
]
#let severity-color(value) = if value == "critical" { critical } else if value == "warning" { warning } else { teal }
#let two-digit(value) = if value < 10 { "0" + str(value) } else { str(value) }
#let metadata(label, value) = [
  #text(font: mono, size: 5.5pt, weight: "bold", tracking: 0.2pt, fill: muted)[#label]
  #linebreak()
  #text(font: mono, size: 7pt, fill: ink)[#value]
]

#block(fill: navy, inset: 15pt, radius: 3pt, width: 100%)[
  #grid(
    columns: (1fr, auto),
    column-gutter: 12pt,
    align: (left + horizon, right + horizon),
    [
      #text(font: mono, size: 6.5pt, weight: "bold", tracking: 0.3pt, fill: acid)[PX4 / FLIGHT INTELLIGENCE]
      #v(11pt)
      #text(size: 23pt, weight: "bold", fill: white)[SECOND-OPINION\\
      FLIGHT BRIEF]
      #v(7pt)
      #text(size: 8pt, fill: rgb("#b7cbcd"))[Evidence-backed AI analysis / #report-id]
    ],
    [
      #box(stroke: 0.7pt + rgb("#6f898d"), inset: (x: 10pt, y: 9pt), radius: 2pt)[
        #align(center)[
          #text(font: mono, size: 6pt, tracking: 0.2pt, fill: rgb("#b7cbcd"))[ASSESSMENT]
          #linebreak()
          #text(size: 15pt, weight: "bold", fill: acid)[#risk-label]
          #if confidence != none [
            #linebreak()
            #text(font: mono, size: 6.5pt, fill: rgb("#b7cbcd"))[#confidence% CONFIDENCE]
          ]
        ]
      ]
    ],
  )
]

#v(12pt)
#eyebrow("00 / EXECUTIVE READOUT")
#v(4pt)
#text(size: 10.5pt, fill: body-ink)[#summary]

#section-title("01 / OBSERVATIONS", "Evidence-backed findings")
#if findings.len() == 0 [
  #block(fill: rgb("#edf6f0"), stroke: 0.6pt + rgb("#9fcab9"), inset: 10pt, width: 100%, radius: 2pt)[
    #text(weight: "bold", fill: rgb("#26634f"))[NO ANOMALY FINDINGS]
    #linebreak()
    #text(size: 8pt, fill: muted)[The supplied evidence did not produce a reportable anomaly.]
  ]
] else {
  for (index, finding) in findings.enumerate() [
    #block(stroke: (left: 2.5pt + severity-color(finding.severity), rest: 0.5pt + line-color), inset: 10pt, width: 100%, radius: 1.5pt, below: 7pt)[
      #grid(columns: (1fr, auto), column-gutter: 8pt,
        [#text(font: mono, size: 6.5pt, weight: "bold", tracking: 0.15pt, fill: severity-color(finding.severity))[#finding.code / #upper(finding.severity) / #upper(finding.category)]],
        [#if finding.time != none { text(font: mono, size: 6.5pt, fill: muted, finding.time) }],
      )
      #v(5pt)
      #text(size: 11pt, weight: "bold")[#two-digit(index + 1)  #finding.title]
      #v(4pt)
      #text(size: 9pt, fill: body-ink)[#finding.explanation]
      #if finding.evidence.len() > 0 [
        #v(6pt)
        #block(fill: soft, inset: 7pt, width: 100%, radius: 1pt)[
          #eyebrow("SUPPORTING EVIDENCE")
          #v(3pt)
          #for evidence in finding.evidence [
            #grid(columns: (7pt, 1fr), column-gutter: 3pt,
              [#text(fill: teal, size: 8pt)[+]],
              [#text(size: 8pt, fill: body-ink)[#evidence]],
            )
            #v(2pt)
          ]
        ]
      ]
    ]
  ]
}

#section-title("02 / NOMINAL SIGNALS", "Positive observations")
#if positives.len() == 0 [
  #text(size: 8.5pt, fill: muted)[No positive observations were returned.]
] else {
  for (index, observation) in positives.enumerate() [
    #grid(columns: (19pt, 1fr), column-gutter: 7pt,
      [#box(fill: navy, inset: (x: 5pt, y: 3pt), radius: 1pt)[#text(font: mono, size: 6pt, weight: "bold", fill: acid)[#two-digit(index + 1)]]],
      [#text(size: 9pt, fill: body-ink)[#observation]],
    )
    #v(5pt)
  ]
}

#section-title("03 / NEXT ACTIONS", "Recommended follow-up")
#if recommendations.len() == 0 [
  #text(size: 8.5pt, fill: muted)[No follow-up actions were returned.]
] else {
  for (index, recommendation) in recommendations.enumerate() [
    #block(fill: soft, inset: 9pt, width: 100%, radius: 2pt, below: 6pt)[
      #grid(columns: (24pt, 1fr), column-gutter: 8pt,
        [#box(fill: navy, inset: 6pt, radius: 1pt)[#align(center)[#text(font: mono, size: 7pt, weight: "bold", fill: acid)[#str(index + 1)]]]],
        [
          #eyebrow(recommendation.priority + " PRIORITY")
          #linebreak()
          #text(size: 10pt, weight: "bold")[#recommendation.action]
          #v(3pt)
          #text(size: 8.5pt, fill: body-ink)[#recommendation.rationale]
        ],
      )
    ]
  ]
}

#if limitations.len() > 0 [
  #section-title("04 / ANALYSIS ENVELOPE", "Known limitations")
  #block(stroke: 0.6pt + line-color, inset: 9pt, width: 100%, radius: 2pt)[
    #for limitation in limitations [
      #grid(columns: (7pt, 1fr), column-gutter: 4pt,
        [#text(fill: warning, "!")],
        [#text(size: 8.5pt, fill: body-ink)[#limitation]],
      )
      #v(3pt)
    ]
  ]
]

#v(12pt)
#line(length: 100%, stroke: 0.6pt + line-color)
#v(6pt)
#grid(
  columns: (1fr, 1fr, 0.7fr),
  column-gutter: 12pt,
  metadata("MODEL", model),
  metadata("UTC GENERATED", generated-at),
  metadata("SCHEMA / TOKENS", str(schema-version) + " / " + if token-load == none { "N/A" } else { str(token-load) }),
)
#v(10pt)
#block(fill: rgb("#fff3df"), inset: 8pt, width: 100%, radius: 2pt)[
  #text(font: mono, size: 6.5pt, weight: "bold", tracking: 0.2pt, fill: warning)[ENGINEERING AID]
  #linebreak()
  #text(size: 7.5pt, fill: rgb("#5c4a2f"))[AI output is not an airworthiness determination. Verify findings against plots, messages, and deterministic diagnostics.]
]
`;
}

export function aiAnalysisPdfFilename(reportId?: string, generatedAt?: string): string {
  const id = cleanText(reportId?.trim() || 'flight').replace(/[^a-zA-Z0-9_-]+/g, '-').replace(/^-+|-+$/g, '') || 'flight';
  const date = generatedAt && !Number.isNaN(new Date(generatedAt).getTime())
    ? new Date(generatedAt).toISOString().slice(0, 10)
    : new Date().toISOString().slice(0, 10);
  return `flight-review-${id}-ai-analysis-${date}.pdf`;
}

async function getTypstCompiler(): Promise<TypstModule['$typst']> {
  if (!typstPromise) {
    typstPromise = (async () => {
      const [typst, wasm, monoRegular, monoBold, sansRegular, sansBold] = await Promise.all([
        import('@myriaddreamin/typst.ts'),
        import('@myriaddreamin/typst-ts-web-compiler/wasm?url'),
        import('dejavu-fonts-ttf/ttf/DejaVuSansMono.ttf?url'),
        import('dejavu-fonts-ttf/ttf/DejaVuSansMono-Bold.ttf?url'),
        import('dejavu-fonts-ttf/ttf/DejaVuSans.ttf?url'),
        import('dejavu-fonts-ttf/ttf/DejaVuSans-Bold.ttf?url'),
      ]);

      typst.$typst.setCompilerInitOptions({
        getModule: () => wasm.default,
        beforeBuild: [typst.loadFonts([
          monoRegular.default,
          monoBold.default,
          sansRegular.default,
          sansBold.default,
        ], { assets: false })],
      });
      return typst.$typst;
    })();
  }
  return typstPromise;
}

export async function generateAiAnalysisPdf(analysis: AiAnalysis, reportId?: string): Promise<Uint8Array> {
  const compiler = await getTypstCompiler();
  const pdf = await compiler.pdf({ mainContent: buildAiAnalysisTypst(analysis, reportId) });
  if (!pdf || pdf.byteLength === 0) throw new Error('The PDF compiler returned an empty document.');
  return pdf;
}

export async function downloadAiAnalysisPdf(analysis: AiAnalysis, reportId?: string): Promise<void> {
  const pdf = await generateAiAnalysisPdf(analysis, reportId);
  const data = Uint8Array.from(pdf).buffer;
  const url = URL.createObjectURL(new Blob([data], { type: PDF_MIME_TYPE }));
  const anchor = document.createElement('a');
  anchor.href = url;
  anchor.download = aiAnalysisPdfFilename(reportId, analysis.generated_at);
  anchor.click();
  window.setTimeout(() => URL.revokeObjectURL(url), 1_000);
}
