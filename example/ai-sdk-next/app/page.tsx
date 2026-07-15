"use client";

import { FormEvent, useState } from "react";

type GenerateResponse = {
  text?: string;
  usage?: unknown;
  finishReason?: string;
  model?: string;
  baseURL?: string;
  latencyMs?: number;
  error?: string;
};

const defaultPrompt = "Explain the concept of quantum entanglement.";
const defaultBaseURL = "http://127.0.0.1:17777/v1";

export default function Page() {
  const [apiKey, setApiKey] = useState("");
  const [baseURL, setBaseURL] = useState(defaultBaseURL);
  const [prompt, setPrompt] = useState(defaultPrompt);
  const [model, setModel] = useState("");
  const [result, setResult] = useState<GenerateResponse | null>(null);
  const [loading, setLoading] = useState(false);

  async function runTest(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setLoading(true);
    setResult(null);

    try {
      const response = await fetch("/api/generate", {
        method: "POST",
        headers: {
          "content-type": "application/json",
        },
        body: JSON.stringify({ apiKey, baseURL, model, prompt }),
      });
      const data = (await response.json()) as GenerateResponse;
      setResult(response.ok ? data : { error: data.error ?? response.statusText });
    } catch (error) {
      setResult({
        error: error instanceof Error ? error.message : "Unexpected request failure",
      });
    } finally {
      setLoading(false);
    }
  }

  return (
    <main className="shell">
      <section className="panel">
        <div>
          <p className="eyebrow">Local AI Gateway</p>
          <h1>AI SDK generateText test</h1>
          <p className="lede">
            This page calls a Next.js route that uses Vercel AI SDK
            generateText against your local OpenAI-compatible gateway.
          </p>
        </div>

        <form onSubmit={runTest} className="form">
          <label>
            <span>Local API Key</span>
            <input
              type="password"
              value={apiKey}
              onChange={(event) => setApiKey(event.target.value)}
              placeholder="Paste local gateway key"
              autoComplete="off"
            />
          </label>
          <label>
            <span>Gateway Base URL</span>
            <input
              value={baseURL}
              onChange={(event) => setBaseURL(event.target.value)}
              placeholder="http://127.0.0.1:17777/v1"
            />
          </label>
          <label>
            <span>Model</span>
            <input
              value={model}
              onChange={(event) => setModel(event.target.value)}
              placeholder="gpt-5.5"
            />
          </label>
          <label>
            <span>Prompt</span>
            <textarea
              rows={5}
              value={prompt}
              onChange={(event) => setPrompt(event.target.value)}
            />
          </label>
          <button disabled={loading}>{loading ? "Running..." : "Run generateText"}</button>
        </form>
      </section>

      <section className="panel output">
        <div className="outputHeader">
          <h2>Response</h2>
          {result?.latencyMs ? <span>{result.latencyMs}ms</span> : null}
        </div>
        {result ? (
          result.error ? (
            <pre className="error">{result.error}</pre>
          ) : (
            <>
              <pre>{result.text}</pre>
              <dl>
                <div>
                  <dt>Model</dt>
                  <dd>{result.model}</dd>
                </div>
                <div>
                  <dt>Base URL</dt>
                  <dd>{result.baseURL}</dd>
                </div>
                <div>
                  <dt>Finish</dt>
                  <dd>{result.finishReason}</dd>
                </div>
              </dl>
              <pre className="usage">{JSON.stringify(result.usage, null, 2)}</pre>
            </>
          )
        ) : (
          <p className="empty">Run the test to see the gateway response.</p>
        )}
      </section>

      <style jsx>{`
        .shell {
          display: grid;
          gap: 24px;
          grid-template-columns: minmax(320px, 0.82fr) minmax(360px, 1.18fr);
          min-height: 100vh;
          padding: 48px;
        }

        .panel {
          align-self: start;
          background: rgba(255, 255, 255, 0.86);
          border: 1px solid rgba(16, 32, 26, 0.12);
          border-radius: 8px;
          box-shadow: 0 24px 70px rgba(16, 32, 26, 0.12);
          padding: 28px;
        }

        .eyebrow {
          color: #087a52;
          font-size: 12px;
          font-weight: 800;
          letter-spacing: 0.16em;
          margin: 0 0 10px;
          text-transform: uppercase;
        }

        h1,
        h2 {
          letter-spacing: 0;
          margin: 0;
        }

        h1 {
          font-size: 36px;
          line-height: 1;
        }

        h2 {
          font-size: 18px;
        }

        .lede {
          color: #52645b;
          line-height: 1.6;
          margin: 16px 0 0;
        }

        .form {
          display: grid;
          gap: 18px;
          margin-top: 28px;
        }

        label {
          display: grid;
          gap: 8px;
        }

        label span {
          color: #52645b;
          font-size: 12px;
          font-weight: 800;
          letter-spacing: 0.12em;
          text-transform: uppercase;
        }

        input,
        textarea {
          background: #f8fbf9;
          border: 1px solid rgba(16, 32, 26, 0.16);
          border-radius: 8px;
          color: #10201a;
          outline: none;
          padding: 12px 14px;
          width: 100%;
        }

        textarea {
          resize: vertical;
        }

        input:focus,
        textarea:focus {
          border-color: #087a52;
          box-shadow: 0 0 0 3px rgba(8, 122, 82, 0.12);
        }

        button {
          background: #10201a;
          border: 0;
          border-radius: 8px;
          color: white;
          font-weight: 800;
          padding: 13px 16px;
        }

        button:disabled {
          cursor: wait;
          opacity: 0.68;
        }

        .output {
          min-height: calc(100vh - 96px);
        }

        .outputHeader {
          align-items: center;
          display: flex;
          justify-content: space-between;
          gap: 12px;
          margin-bottom: 18px;
        }

        .outputHeader span {
          background: #e8f3ee;
          border-radius: 999px;
          color: #087a52;
          font-size: 12px;
          font-weight: 800;
          padding: 4px 10px;
        }

        pre {
          background: #10201a;
          border-radius: 8px;
          color: #e8fff5;
          line-height: 1.55;
          margin: 0;
          overflow: auto;
          padding: 18px;
          white-space: pre-wrap;
        }

        .error {
          background: #fff1f1;
          color: #9f1d1d;
        }

        .usage {
          background: #f4f7f5;
          color: #10201a;
          margin-top: 14px;
        }

        dl {
          display: grid;
          gap: 10px;
          grid-template-columns: repeat(3, minmax(0, 1fr));
          margin: 16px 0 0;
        }

        dl div {
          background: #f4f7f5;
          border-radius: 8px;
          padding: 12px;
        }

        dt {
          color: #6a7b72;
          font-size: 11px;
          font-weight: 800;
          letter-spacing: 0.12em;
          text-transform: uppercase;
        }

        dd {
          margin: 6px 0 0;
          overflow-wrap: anywhere;
        }

        .empty {
          color: #6a7b72;
          margin: 0;
        }

        @media (max-width: 880px) {
          .shell {
            grid-template-columns: 1fr;
            padding: 20px;
          }

          .output {
            min-height: auto;
          }

          dl {
            grid-template-columns: 1fr;
          }
        }
      `}</style>
    </main>
  );
}
