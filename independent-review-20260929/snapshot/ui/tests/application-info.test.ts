import { beforeEach, describe, expect, it, vi } from "vitest";
import { invoke, isTauri } from "@tauri-apps/api/core";
import {
  decodeApplicationInfo,
  decodeWorkUnitIds,
  readApplicationInfo,
  readWorkUnitIds
} from "../src/application-info";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn(), isTauri: vi.fn() }));

const valid = { version: "0.1.0", buildStage: "work_graph", workUnitsCount: 9 } as const;

describe("closed application metadata", () => {
  it("accepts the known three-field work_graph response", () => {
    expect(decodeApplicationInfo(valid)).toEqual(valid);
  });

  it("accepts the legacy empty_shell stage for backward compatibility", () => {
    const legacy = { version: "0.1.0", buildStage: "empty_shell", workUnitsCount: 0 } as const;
    expect(decodeApplicationInfo(legacy)).toEqual(legacy);
  });

  it.each([
    null, undefined, [], "0.1.0", {},
    { version: "0.1.0" },
    { version: "0.1.0", buildStage: "work_graph" },
    { ...valid, version: 1 },
    { ...valid, version: "<script>1.0.0</script>" },
    { ...valid, version: "1.0.0\n" },
    { ...valid, version: "9".repeat(33) + ".0.0" },
    { ...valid, buildStage: "ready" },
    { ...valid, buildStage: 1 },
    { ...valid, workUnitsCount: -1 },
    { ...valid, workUnitsCount: 1.5 },
    { ...valid, workUnitsCount: "9" },
    { ...valid, workUnitsCount: 1001 },
    { ...valid, secret: "must-not-be-exposed" },
    { ...valid, canExecuteWork: true }
  ])("rejects unknown or malformed data: %j", (value) => {
    expect(decodeApplicationInfo(value)).toBeUndefined();
  });
});

describe("the only read-only native metadata command", () => {
  beforeEach(() => { vi.resetAllMocks(); });

  it("does not invent a native version in a browser", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    expect(await readApplicationInfo()).toEqual({ kind: "unavailable", reason: "not_desktop" });
    expect(invoke).not.toHaveBeenCalled();
  });

  it("invokes exactly application_info without any input", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue(valid);
    expect(await readApplicationInfo()).toEqual({ kind: "available", info: valid });
    expect(invoke).toHaveBeenCalledExactlyOnceWith("application_info");
  });

  it("fails closed on native command refusal without exposing raw errors", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockRejectedValue(new Error("private native detail"));
    expect(await readApplicationInfo()).toEqual({ kind: "unavailable", reason: "native_unavailable" });
  });

  it("does not render a newer unknown build stage as readiness", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue({ ...valid, buildStage: "work_ready" });
    expect(await readApplicationInfo()).toEqual({ kind: "unavailable", reason: "invalid_response" });
  });
});

describe("work unit id list decoder", () => {
  it("accepts a closed list of WU-NNN strings", () => {
    expect(decodeWorkUnitIds(["WU-001", "WU-009"])).toEqual(["WU-001", "WU-009"]);
    expect(decodeWorkUnitIds([])).toEqual([]);
  });

  it.each([
    null, undefined, {}, "WU-001",
    [null], [undefined], [1], [{}],
    ["wu-001"], ["WU-1"], ["WU-0001"], ["WU-001a"], [" WU-001"],
    ["../../etc/passwd"], ["<script>"],
    Array.from({ length: 1001 }, (_, i) => `WU-${String(i).padStart(3, "0")}`)
  ])("rejects malformed data: %j", (value) => {
    expect(decodeWorkUnitIds(value)).toBeUndefined();
  });
});

describe("readWorkUnitIds", () => {
  beforeEach(() => { vi.resetAllMocks(); });

  it("does not invent work units in a browser", async () => {
    vi.mocked(isTauri).mockReturnValue(false);
    expect(await readWorkUnitIds()).toEqual({ kind: "unavailable", reason: "not_desktop" });
    expect(invoke).not.toHaveBeenCalled();
  });

  it("invokes exactly work_unit_ids with no input and returns the validated list", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue(["WU-001", "WU-002"]);
    expect(await readWorkUnitIds()).toEqual({
      kind: "available",
      unitIds: ["WU-001", "WU-002"]
    });
    expect(invoke).toHaveBeenCalledExactlyOnceWith("work_unit_ids");
  });

  it("fails closed on native command refusal without exposing raw errors", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockRejectedValue(new Error("private native detail"));
    expect(await readWorkUnitIds()).toEqual({ kind: "unavailable", reason: "native_unavailable" });
  });

  it("rejects a malformed list from native code", async () => {
    vi.mocked(isTauri).mockReturnValue(true);
    vi.mocked(invoke).mockResolvedValue(["WU-001", "not-a-unit"]);
    expect(await readWorkUnitIds()).toEqual({ kind: "unavailable", reason: "invalid_response" });
  });
});
