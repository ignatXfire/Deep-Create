import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it, vi } from "vitest";
import { App } from "../src/App";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
  isTauri: vi.fn(() => false)
}));

describe("stage-one version page", () => {
  it("states its actual limits and owner-only write controls", () => {
    const page = renderToStaticMarkup(<App />);
    expect(page).toContain("Deep Create cannot execute work yet");
    expect(page).toContain("No models are running");
    expect(page).toContain("Separate from Hermes");
    // Write path is present but owner-gated; no progress bars or fake execution indicators
    expect(page).not.toMatch(/<progress|<iframe/);
    expect(page).not.toContain("0.1.0");
  });

  it("shows a work-graph section with a reading state", () => {
    const page = renderToStaticMarkup(<App />);
    expect(page).toContain("Work graph");
    expect(page).toContain("Reading work graph…");
  });

  it("labels the work graph as a static development fixture, not live state", () => {
    const page = renderToStaticMarkup(<App />);
    expect(page).toContain("Демонстрационный набор данных");
    expect(page).toContain("Не является актуальным состоянием системы");
  });
});
