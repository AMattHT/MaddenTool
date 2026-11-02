import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { StrictMode } from "react";
import { describe, expect, it, vi } from "vitest";
import type { AppInfo } from "./api/app-info";
import App, { LOAD_TIMEOUT_MS } from "./App";

const info: AppInfo = {
  appName: "MaddenTool",
  appVersion: "9.8.7",
  game: "Madden 27",
  platform: "linux",
  arch: "aarch64",
};

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason: unknown) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

describe("startup view", () => {
  it("startup_displays_native_information", async () => {
    render(<App loadAppInfo={() => Promise.resolve(info)} />);

    expect(await screen.findByRole("heading", { name: "MaddenTool" })).toBeVisible();
    expect(screen.getByText("Madden 27")).toBeVisible();
    expect(screen.getByText("9.8.7")).toBeVisible();
    expect(screen.getByText("linux")).toBeVisible();
    expect(screen.getByText("aarch64")).toBeVisible();
  });

  it("announces a readable loading state until the backend answers", async () => {
    const pending = deferred<AppInfo>();
    render(<App loadAppInfo={() => pending.promise} />);

    expect(screen.getByRole("status")).toHaveTextContent(/connecting to the native backend/i);
    await act(async () => pending.resolve(info));
    expect(screen.queryByText(/connecting/i)).not.toBeInTheDocument();
  });

  it("failed_command_can_retry", async () => {
    const user = userEvent.setup();
    const load = vi
      .fn<() => Promise<AppInfo>>()
      .mockRejectedValueOnce(new Error("ipc down"))
      .mockResolvedValueOnce(info);
    render(<App loadAppInfo={load} />);

    expect(await screen.findByRole("alert")).toHaveTextContent("ipc down");
    expect(screen.queryByRole("status")).not.toBeInTheDocument();
    expect(screen.queryByText("9.8.7")).not.toBeInTheDocument();

    const retry = screen.getByRole("button", { name: "Retry" });
    retry.focus();
    await user.keyboard("{Enter}");

    expect(await screen.findByText("9.8.7")).toBeVisible();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(load).toHaveBeenCalledTimes(2);
  });

  it("a backend that never answers reaches the retryable error", async () => {
    vi.useFakeTimers();
    try {
      render(<App loadAppInfo={() => new Promise<AppInfo>(() => {})} />);
      await act(async () => vi.advanceTimersByTime(LOAD_TIMEOUT_MS));

      expect(screen.getByRole("alert")).toHaveTextContent(/did not respond/i);
      expect(screen.getByRole("button", { name: "Retry" })).toBeVisible();
      expect(screen.queryByRole("status")).not.toBeInTheDocument();
    } finally {
      vi.useRealTimers();
    }
  });

  it("describes a non-Error rejection instead of showing nothing", async () => {
    render(<App loadAppInfo={() => Promise.reject("command not allowed")} />);
    expect(await screen.findByRole("alert")).toHaveTextContent("command not allowed");
  });

  it("unmounted_view_ignores_late_result", async () => {
    const consoleError = vi.spyOn(console, "error");
    const pending = deferred<AppInfo>();
    const { unmount } = render(<App loadAppInfo={() => pending.promise} />);

    unmount();
    await act(async () => pending.resolve(info));

    expect(screen.queryByText("9.8.7")).not.toBeInTheDocument();
    expect(consoleError).not.toHaveBeenCalled();
  });

  // StrictMode mounts, cleans up and remounts the effect, so two attempts overlap.
  // This is the case that fails if an abandoned attempt can still set state.
  it.each([
    [
      "a late failure",
      (first: ReturnType<typeof deferred<AppInfo>>) => first.reject(new Error("late failure")),
    ],
    [
      "a late success",
      (first: ReturnType<typeof deferred<AppInfo>>) =>
        first.resolve({ ...info, appVersion: "0.0.1" }),
    ],
  ])("an abandoned attempt cannot overwrite the current one: %s", async (_name, settleFirst) => {
    const first = deferred<AppInfo>();
    const second = deferred<AppInfo>();
    const load = vi
      .fn<() => Promise<AppInfo>>()
      .mockReturnValueOnce(first.promise)
      .mockReturnValueOnce(second.promise);
    render(
      <StrictMode>
        <App loadAppInfo={load} />
      </StrictMode>,
    );
    expect(load).toHaveBeenCalledTimes(2);

    await act(async () => second.resolve(info));
    await act(async () => settleFirst(first));

    expect(screen.getByText("9.8.7")).toBeVisible();
    expect(screen.queryByText("0.0.1")).not.toBeInTheDocument();
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("makes no claim of game-file support in this build", async () => {
    render(<App loadAppInfo={() => Promise.resolve(info)} />);
    expect(await screen.findByText(/cannot open game files yet/i)).toBeVisible();
  });
});
