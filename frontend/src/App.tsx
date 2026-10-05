import { useCallback, useState } from 'react';

import { postReview } from '@/api/client';
import type { ReviewResult } from '@/api/types';

const SAMPLES = [
  'Does aspirin prevent recurrent stroke?',
  'What is the evidence for SGLT2 inhibitors in heart failure?',
  'Melatonin for sleep onset in adults?',
];

export default function App() {
  const [question, setQuestion] = useState(SAMPLES[0] ?? '');
  const [result, setResult] = useState<ReviewResult | null>(null);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const run = useCallback(async (value: string) => {
    if (!value.trim()) return;
    setBusy(true);
    setError(null);
    setResult(null);
    try {
      setResult(await postReview(value));
    } catch (err) {
      setError(String(err));
    } finally {
      setBusy(false);
    }
  }, []);

  return (
    <div className="flex h-full flex-col">
      <header className="border-b border-line bg-panel px-4 py-3">
        <h1 className="text-base font-semibold">AI Literature Review</h1>
        <p className="text-xs text-muted">
          PubMed search → TF-IDF relevance ranking → synthesis with
          citations. Evidence summary — not medical advice.
        </p>
      </header>

      <main className="mx-auto flex w-full max-w-4xl flex-1 flex-col gap-4 overflow-y-auto p-4">
        <section className="rounded-lg border border-line bg-panel p-4">
          <label className="mb-2 block text-sm font-semibold">
            Clinical question
          </label>
          <textarea
            value={question}
            onChange={(e) => setQuestion(e.target.value)}
            rows={2}
            className="w-full rounded border border-line p-2 text-sm"
          />
          <div className="mt-2 flex flex-wrap items-center gap-2">
            <button
              type="button"
              onClick={() => void run(question)}
              disabled={busy || !question.trim()}
              className="rounded bg-primary px-4 py-1.5 text-sm font-medium text-white disabled:opacity-50"
            >
              {busy ? 'Searching PubMed…' : 'Review evidence'}
            </button>
            {SAMPLES.map((sample) => (
              <button
                key={sample}
                type="button"
                onClick={() => setQuestion(sample)}
                className="rounded border border-line px-2 py-1 text-xs text-muted hover:bg-surface"
              >
                {sample}
              </button>
            ))}
          </div>
        </section>

        {error && (
          <p className="rounded border border-red-300 bg-red-50 p-3 text-sm text-red-800">
            {error}
          </p>
        )}

        {result && (
          <>
            <section className="rounded-lg border border-line bg-panel p-4">
              <h2 className="mb-2 text-sm font-semibold">Synthesis</h2>
              <p className="whitespace-pre-line text-sm">{result.synthesis}</p>
              <p className="mt-3 border-t border-line pt-2 text-xs text-muted">
                Confidence: {result.confidence} · {result.articles_found}{' '}
                articles found · model:{' '}
                {result.stub_model ? 'stub' : 'ollama'}
              </p>
            </section>

            <section className="flex flex-col gap-2">
              <h2 className="text-sm font-semibold">Citations</h2>
              {result.citations.map((citation) => (
                <article
                  key={citation.pmid}
                  className="rounded-lg border border-line bg-panel p-3"
                >
                  <a
                    href={citation.pmid}
                    target="_blank"
                    rel="noreferrer"
                    className="text-sm font-medium text-primary underline"
                  >
                    {citation.title}
                  </a>
                  <p className="mt-1 text-xs text-muted">
                    {citation.journal} · {citation.year} · relevance{' '}
                    {citation.relevance}
                  </p>
                  {citation.evidence && (
                    <p className="mt-1 line-clamp-3 text-xs">{citation.evidence}</p>
                  )}
                </article>
              ))}
            </section>

            <p className="text-[10px] italic text-muted">
              {result.disclaimer}
            </p>
          </>
        )}
      </main>
    </div>
  );
}
