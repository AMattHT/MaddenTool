import { useEffect, useState } from "react";
import type { AppInfo } from "./api/app-info";

/** A backend that never answers must still reach the recoverable error state. */
export const LOAD_TIMEOUT_MS = 10_000;

type View =
  { status: "loading" } | { status: "error"; message: string } | { status: "ready"; info: AppInfo };

function describeError(reason: unknown): string {
  if (reason instanceof Error && reason.message) return reason.message;
  if (typeof reason === "string" && reason) return reason;
  return "The native backend did not return an error description.";
}

export default function App({ loadAppInfo }: { loadAppInfo: () => Promise<AppInfo> }) {
  const [view, setView] = useState<View>({ status: "loading" });
  const [attempt, setAttempt] = useState(0);

  useEffect(() => {
    let settled = false;
    const settle = (next: View) => {
      if (settled) return;
      settled = true;
      setView(next);
    };

    const timeout = setTimeout(() => {
      settle({ status: "error", message: "The native backend did not respond in time." });
    }, LOAD_TIMEOUT_MS);

    loadAppInfo()
      .then(
        (info) => settle({ status: "ready", info }),
        (reason: unknown) => settle({ status: "error", message: describeError(reason) }),
      )
      .finally(() => clearTimeout(timeout));

    return () => {
      settled = true;
      clearTimeout(timeout);
    };
  }, [loadAppInfo, attempt]);

  const retry = () => {
    setView({ status: "loading" });
    setAttempt((current) => current + 1);
  };

  return (
    <main className="startup">
      <h1>{view.status === "ready" ? view.info.appName : "MaddenTool"}</h1>
      {view.status === "loading" && (
        <p role="status" className="startup__status">
          <span className="startup__spinner" aria-hidden="true" />
          Connecting to the native backend…
        </p>
      )}
      {view.status === "error" && (
        <div role="alert" className="startup__error">
          <h2>The native backend could not be reached</h2>
          <p>{view.message}</p>
          <button type="button" onClick={retry}>
            Retry
          </button>
        </div>
      )}
      {view.status === "ready" && (
        <>
          <dl className="startup__facts">
            <div>
              <dt>Game</dt>
              <dd>{view.info.game}</dd>
            </div>
            <div>
              <dt>Version</dt>
              <dd>{view.info.appVersion}</dd>
            </div>
            <div>
              <dt>Platform</dt>
              <dd>{view.info.platform}</dd>
            </div>
            <div>
              <dt>Architecture</dt>
              <dd>{view.info.arch}</dd>
            </div>
          </dl>
          <p className="startup__note">
            Reported by the Rust backend. This foundation build cannot open game files yet.
          </p>
        </>
      )}
    </main>
  );
}
