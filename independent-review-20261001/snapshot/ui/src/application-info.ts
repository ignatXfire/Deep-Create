import { invoke, isTauri } from "@tauri-apps/api/core";

export type ApplicationInfo = Readonly<{
  version: string;
  buildStage: "empty_shell" | "work_graph";
  workUnitsCount: number;
}>;

export type ObjectiveDraft = Readonly<{
  statement: string;
  completionCriteria: readonly string[];
}>;

export type WriteResult =
  | Readonly<{ kind: "success"; response: string }>
  | Readonly<{ kind: "error"; reason: "not_desktop" | "native_unavailable" | "invalid_response" | "invalid_input" }>;

export type PlanResult =
  | Readonly<{ kind: "planned"; graphJson: string }>
  | Readonly<{ kind: "error"; reason: "not_desktop" | "native_unavailable" | "invalid_response" | "invalid_input" }>;

export type InfoResult =
  | Readonly<{ kind: "available"; info: ApplicationInfo }>
  | Readonly<{ kind: "unavailable"; reason: "not_desktop" | "native_unavailable" | "invalid_response" }>;

/** C-14 / stage 1: validate untrusted IPC as a closed response, never a work state. */
export function decodeApplicationInfo(value: unknown): ApplicationInfo | undefined {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return undefined;
  if (
    Object.keys(value).length !== 3 ||
    !("version" in value) ||
    !("buildStage" in value) ||
    !("workUnitsCount" in value)
  ) return undefined;
  if (
    typeof value.version !== "string" ||
    !/^\d+\.\d+\.\d+$/.test(value.version) ||
    value.version.trim() !== value.version ||
    value.version.length > 32
  ) return undefined;
  if (value.buildStage !== "empty_shell" && value.buildStage !== "work_graph") return undefined;
  if (
    typeof value.workUnitsCount !== "number" ||
    !Number.isInteger(value.workUnitsCount) ||
    value.workUnitsCount < 0 ||
    value.workUnitsCount > 1000
  ) return undefined;
  return {
    version: value.version,
    buildStage: value.buildStage,
    workUnitsCount: value.workUnitsCount
  };
}

/** The only native call available to this page; no inputs or write side effects. */
export async function readApplicationInfo(): Promise<InfoResult> {
  if (!isTauri()) return { kind: "unavailable", reason: "not_desktop" };
  let response: unknown;
  try {
    response = await invoke<unknown>("application_info");
  } catch {
    return { kind: "unavailable", reason: "native_unavailable" };
  }
  const info = decodeApplicationInfo(response);
  return info === undefined
    ? { kind: "unavailable", reason: "invalid_response" }
    : { kind: "available", info };
}

// ---------------------------------------------------------------------------
// Work graph (read-only, no authority)

export type WorkUnitIdsResult =
  | Readonly<{ kind: "available"; unitIds: readonly string[] }>
  | Readonly<{ kind: "unavailable"; reason: "not_desktop" | "native_unavailable" | "invalid_response" }>;

/** Validate the work-unit list as a closed array of bounded ASCII IDs. */
export function decodeWorkUnitIds(value: unknown): readonly string[] | undefined {
  if (!Array.isArray(value)) return undefined;
  if (value.length > 1000) return undefined;
  const ids: string[] = [];
  for (const item of value) {
    if (typeof item !== "string") return undefined;
    if (!/^WU-\d{3}$/.test(item)) return undefined;
    ids.push(item);
  }
  return ids;
}

/** Read the current work-unit IDs from the desktop shell. */
export async function readWorkUnitIds(): Promise<WorkUnitIdsResult> {
  if (!isTauri()) return { kind: "unavailable", reason: "not_desktop" };
  let response: unknown;
  try {
    response = await invoke<unknown>("work_unit_ids");
  } catch {
    return { kind: "unavailable", reason: "native_unavailable" };
  }
  const ids = decodeWorkUnitIds(response);
  return ids === undefined
    ? { kind: "unavailable", reason: "invalid_response" }
    : { kind: "available", unitIds: ids };
}

// ---------------------------------------------------------------------------
// Work unit details (read-only)

export type WorkUnit = Readonly<{
  id: string;
  kind: string;
  stage: string;
  summary: string;
  verdict: string | null;
  artifacts: readonly string[];
}>;

export type WorkUnitsResult =
  | Readonly<{ kind: "available"; units: readonly WorkUnit[] }>
  | Readonly<{ kind: "unavailable"; reason: "not_desktop" | "native_unavailable" | "invalid_response" }>;

/** Validate a single work unit as a closed object. */
export function decodeWorkUnit(value: unknown): WorkUnit | undefined {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return undefined;
  const keys = Object.keys(value);
  if (keys.length !== 6) return undefined;
  if (!("id" in value) || !("kind" in value) || !("stage" in value) || !("summary" in value) || !("verdict" in value) || !("artifacts" in value)) return undefined;
  if (typeof value.id !== "string" || typeof value.kind !== "string" || typeof value.stage !== "string" || typeof value.summary !== "string") return undefined;
  if (value.verdict !== null && typeof value.verdict !== "string") return undefined;
  if (!Array.isArray(value.artifacts)) return undefined;
  const artifacts: string[] = [];
  for (const a of value.artifacts) {
    if (typeof a !== "string") return undefined;
    artifacts.push(a);
  }
  return {
    id: value.id,
    kind: value.kind,
    stage: value.stage,
    summary: value.summary,
    verdict: value.verdict,
    artifacts,
  };
}

/** Read full work unit details from the desktop shell. */
export async function readWorkUnits(): Promise<WorkUnitsResult> {
  if (!isTauri()) return { kind: "unavailable", reason: "not_desktop" };
  let response: unknown;
  try {
    response = await invoke<unknown>("work_units");
  } catch {
    return { kind: "unavailable", reason: "native_unavailable" };
  }
  if (!Array.isArray(response)) return { kind: "unavailable", reason: "invalid_response" };
  const units: WorkUnit[] = [];
  for (const item of response) {
    const unit = decodeWorkUnit(item);
    if (unit === undefined) return { kind: "unavailable", reason: "invalid_response" };
    units.push(unit);
  }
  return { kind: "available", units };
}

// ---------------------------------------------------------------------------
// Write path (owner commands)

function encodeObjectiveDraft(draft: ObjectiveDraft): string {
  // EntityId requires 32 lowercase hex chars (UUIDv7 without dashes)
  // Use fixed valid UUIDv7 values for the singleton scope
  return JSON.stringify({
    scope: {
      session_id: "00000000000070008000000000000001",
      work_id: "00000000000070008000000000000002",
      objective_id: "00000000000070008000000000000003",
      objective_revision: 1,
    },
    statement: draft.statement,
    owner_event_ids: [],
    obligations: [],
    completion_criteria: draft.completionCriteria.map((_, i) =>
      `00000000000070008000000000000${(0x100 + i).toString(16).padStart(3, "0")}`
    ),
    row_version: 1,
  });
}

/** Submit an objective to dc-core via the UI write path. */
export async function submitObjective(draft: ObjectiveDraft): Promise<WriteResult> {
  if (!isTauri()) return { kind: "error", reason: "not_desktop" };
  if (draft.statement.trim().length === 0 || draft.completionCriteria.length === 0) {
    return { kind: "error", reason: "invalid_input" };
  }
  try {
    const response = await invoke<string>("submit_objective", {
      objective_json: encodeObjectiveDraft(draft),
    });
    return { kind: "success", response };
  } catch {
    return { kind: "error", reason: "native_unavailable" };
  }
}

/** Plan a graph from an objective (read-only, no state change). */
export async function planObjectiveGraph(draft: ObjectiveDraft): Promise<PlanResult> {
  if (!isTauri()) return { kind: "error", reason: "not_desktop" };
  if (draft.statement.trim().length === 0 || draft.completionCriteria.length === 0) {
    return { kind: "error", reason: "invalid_input" };
  }
  try {
    const response = await invoke<string>("plan_objective_graph", {
      objective_json: encodeObjectiveDraft(draft),
    });
    return { kind: "planned", graphJson: response };
  } catch {
    return { kind: "error", reason: "native_unavailable" };
  }
}

/** Accept a previously planned graph (owner review write path). */
export async function acceptGraph(graphJson: string): Promise<WriteResult> {
  if (!isTauri()) return { kind: "error", reason: "not_desktop" };
  try {
    const response = await invoke<string>("accept_graph", { graph_json: graphJson });
    return { kind: "success", response };
  } catch {
    return { kind: "error", reason: "native_unavailable" };
  }
}

/** Start a work unit attempt. */
export async function startAttempt(unitId: string): Promise<WriteResult> {
  if (!isTauri()) return { kind: "error", reason: "not_desktop" };
  // EntityId is 32 lowercase hex chars (UUIDv7 without dashes)
  if (!/^[0-9a-f]{32}$/.test(unitId)) return { kind: "error", reason: "invalid_input" };
  try {
    const response = await invoke<string>("start_attempt", {
      unit_id: unitId,
    });
    return { kind: "success", response };
  } catch {
    return { kind: "error", reason: "native_unavailable" };
  }
}

/** Execute the accepted graph. */
export async function executeGraph(): Promise<WriteResult> {
  if (!isTauri()) return { kind: "error", reason: "not_desktop" };
  try {
    const response = await invoke<string>("execute_graph");
    return { kind: "success", response };
  } catch {
    return { kind: "error", reason: "native_unavailable" };
  }
}
