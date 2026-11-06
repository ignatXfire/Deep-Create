import { useEffect, useState } from "react";
import { readApplicationInfo, readWorkUnits, submitObjective, planObjectiveGraph, acceptGraph, startAttempt, executeGraph } from "./application-info";
import type { ApplicationInfo, InfoResult, WorkUnitsResult, WorkUnit, ObjectiveDraft } from "./application-info";

type MetadataState = Readonly<{ kind: "reading" }> | InfoResult;
type WorkUnitsState = Readonly<{ kind: "reading" }> | WorkUnitsResult;

type WriteState = Readonly<
  | { kind: "idle" }
  | { kind: "submitting" }
  | { kind: "success"; response: string }
  | { kind: "error"; reason: string }
>;

type PlanState = Readonly<
  | { kind: "idle" }
  | { kind: "planning" }
  | { kind: "planned"; graphJson: string }
  | { kind: "error"; reason: string }
>;

const buildStageLabels = {
  empty_shell: "Minimal shell",
  work_graph: "Shell + work graph"
} satisfies Record<ApplicationInfo["buildStage"], string>;

function Metadata({ state }: { state: MetadataState }) {
  switch (state.kind) {
    case "reading":
      return <p className="metadata-message" role="status">Reading native build information…</p>;
    case "available":
      return (
        <dl className="build-metadata">
          <div><dt>Application version</dt><dd>{state.info.version}</dd></div>
          <div><dt>Build stage</dt><dd>{buildStageLabels[state.info.buildStage]}</dd></div>
          <div><dt>Work units in graph</dt><dd>{state.info.workUnitsCount}</dd></div>
        </dl>
      );
    case "unavailable": {
      const messages = {
        not_desktop: "Opened in a browser. The native application is unavailable; the version is unverified.",
        native_unavailable: "Could not read native application information. The version is unverified.",
        invalid_response: "The native application returned an unknown information format. The version is unverified."
      } satisfies Record<typeof state.reason, string>;
      return <p className="metadata-message error" role="alert">{messages[state.reason]}</p>;
    }
  }
}

function WorkUnitCard({ unit, onStartAttempt, writeState }: {
  unit: WorkUnit;
  onStartAttempt: (unitId: string) => void;
  writeState: WriteState;
}) {
  return (
    <li className="work-unit-item">
      <div className="work-unit-header">
        <code>{unit.id}</code>
        <span className={`stage-badge stage-${unit.stage}`}>{unit.stage}</span>
      </div>
      <p className="work-unit-summary">{unit.summary}</p>
      <p className="work-unit-kind">Type: {unit.kind}</p>
      {unit.verdict && (
        <p className="work-unit-verdict">Verdict: {unit.verdict}</p>
      )}
      {unit.artifacts.length > 0 && (
        <details className="work-unit-artifacts">
          <summary>Artifacts ({unit.artifacts.length})</summary>
          <ul>
            {unit.artifacts.map((a, i) => <li key={i}><code>{a}</code></li>)}
          </ul>
        </details>
      )}
      <StartAttemptButton
        unitId={unit.id}
        onStart={onStartAttempt}
        writeState={writeState}
      />
    </li>
  );
}

function WorkUnits({ state, onStartAttempt, writeState }: {
  state: WorkUnitsState;
  onStartAttempt: (unitId: string) => void;
  writeState: WriteState;
}) {
  switch (state.kind) {
    case "reading":
      return <p className="metadata-message" role="status">Reading work graph…</p>;
    case "available":
      if (state.units.length === 0) {
        return <p className="metadata-message">The work graph is empty — no work units have been committed yet.</p>;
      }
      return (
        <ul className="work-unit-list" aria-label="Work unit list">
          {state.units.map((unit) => (
            <WorkUnitCard
              key={unit.id}
              unit={unit}
              onStartAttempt={onStartAttempt}
              writeState={writeState}
            />
          ))}
        </ul>
      );
    case "unavailable": {
      const messages = {
        not_desktop: "Opened in a browser. The work graph is unavailable.",
        native_unavailable: "Could not read the work graph from the native application.",
        invalid_response: "The native application returned an unknown work graph format."
      } satisfies Record<typeof state.reason, string>;
      return <p className="metadata-message error" role="alert">{messages[state.reason]}</p>;
    }
  }
}

function ObjectiveForm({ onPlan, onSubmit, onAccept, onExecute, planState, writeState }: {
  onPlan: (draft: ObjectiveDraft) => void;
  onSubmit: (draft: ObjectiveDraft) => void;
  onAccept: (graphJson: string) => void;
  onExecute: () => void;
  planState: PlanState;
  writeState: WriteState;
}) {
  const [statement, setStatement] = useState("");
  const [criteria, setCriteria] = useState("");

  const draft: ObjectiveDraft = {
    statement,
    completionCriteria: criteria.split("\n").map(s => s.trim()).filter(Boolean),
  };

  return (
    <section className="objective-panel" aria-labelledby="objective-title">
      <div className="panel-heading">
        <span className="small-square" aria-hidden="true" />
        <h2 id="objective-title">New objective</h2>
        <span className="read-only">Draft</span>
      </div>
      <div className="form-group">
        <label htmlFor="statement">Objective statement</label>
        <textarea
          id="statement"
          value={statement}
          onChange={(e) => setStatement(e.target.value)}
          placeholder="What should the system accomplish?"
          rows={3}
        />
      </div>
      <div className="form-group">
        <label htmlFor="criteria">Completion criteria (one per line)</label>
        <textarea
          id="criteria"
          value={criteria}
          onChange={(e) => setCriteria(e.target.value)}
          placeholder="Result A\nResult B"
          rows={3}
        />
      </div>
      <div className="button-row">
        <button
          type="button"
          onClick={() => onPlan(draft)}
          disabled={planState.kind === "planning" || writeState.kind === "submitting"}
        >
          {planState.kind === "planning" ? "Planning…" : "Plan graph"}
        </button>
        <button
          type="button"
          onClick={() => onSubmit(draft)}
          disabled={writeState.kind === "submitting" || planState.kind === "planning"}
        >
          {writeState.kind === "submitting" ? "Submitting…" : "Submit for execution"}
        </button>
        {planState.kind === "planned" && (
          <button
            type="button"
            onClick={() => onAccept(planState.graphJson)}
            disabled={writeState.kind === "submitting"}
          >
            {writeState.kind === "submitting" ? "Accepting…" : "Accept graph"}
          </button>
        )}
        {writeState.kind === "success" && writeState.response?.includes("GraphAccepted") && (
          <button
            type="button"
            onClick={() => onExecute()}
            disabled={false}
          >
            Execute graph
          </button>
        )}
      </div>
      {planState.kind === "planned" && (
        <details className="graph-preview">
          <summary>Work graph (JSON)</summary>
          <pre>{planState.graphJson}</pre>
        </details>
      )}
      {writeState.kind === "success" && (
        <p className="metadata-message success" role="status">Objective accepted: {writeState.response}</p>
      )}
      {writeState.kind === "error" && (
        <p className="metadata-message error" role="alert">Error: {writeState.reason}</p>
      )}
      {planState.kind === "error" && (
        <p className="metadata-message error" role="alert">Planning error: {planState.reason}</p>
      )}
    </section>
  );
}

function StartAttemptButton({ unitId, onStart, writeState }: {
  unitId: string;
  onStart: (unitId: string) => void;
  writeState: WriteState;
}) {
  return (
    <button
      type="button"
      className="start-attempt"
      onClick={() => onStart(unitId)}
      disabled={writeState.kind === "submitting"}
      title={`Start attempt for ${unitId}`}
    >
      ▶
    </button>
  );
}

export function App() {
  const [metadata, setMetadata] = useState<MetadataState>({ kind: "reading" });
  const [workUnits, setWorkUnits] = useState<WorkUnitsState>({ kind: "reading" });
  const [writeState, setWriteState] = useState<WriteState>({ kind: "idle" });
  const [planState, setPlanState] = useState<PlanState>({ kind: "idle" });

  useEffect(() => {
    let active = true;
    void readApplicationInfo().then((result) => {
      if (active) setMetadata(result);
    });
    void readWorkUnits().then((result) => {
      if (active) setWorkUnits(result);
    });
    return () => { active = false; };
  }, []);

  const handlePlan = async (draft: ObjectiveDraft) => {
    setPlanState({ kind: "planning" });
    const result = await planObjectiveGraph(draft);
    if (result.kind === "planned") {
      setPlanState({ kind: "planned", graphJson: result.graphJson });
    } else {
      setPlanState({ kind: "error", reason: result.reason });
    }
  };

  const handleSubmit = async (draft: ObjectiveDraft) => {
    setWriteState({ kind: "submitting" });
    const result = await submitObjective(draft);
    if (result.kind === "success") {
      setWriteState({ kind: "success", response: result.response });
      // Refresh work units after successful submission
      void readWorkUnits().then(setWorkUnits);
    } else {
      setWriteState({ kind: "error", reason: result.reason });
    }
  };

  const handleAcceptGraph = async (graphJson: string) => {
    setWriteState({ kind: "submitting" });
    const result = await acceptGraph(graphJson);
    if (result.kind === "success") {
      setWriteState({ kind: "success", response: result.response });
      void readWorkUnits().then(setWorkUnits);
    } else {
      setWriteState({ kind: "error", reason: result.reason });
    }
  };

  const handleExecute = async () => {
    setWriteState({ kind: "submitting" });
    const result = await executeGraph();
    if (result.kind === "success") {
      setWriteState({ kind: "success", response: result.response });
      void readWorkUnits().then(setWorkUnits);
    } else {
      setWriteState({ kind: "error", reason: result.reason });
    }
  };

  const handleStartAttempt = async (unitId: string) => {
    setWriteState({ kind: "submitting" });
    const result = await startAttempt(unitId);
    if (result.kind === "success") {
      setWriteState({ kind: "success", response: `Attempt started for ${unitId}` });
    } else {
      setWriteState({ kind: "error", reason: result.reason });
    }
  };

  return (
    <main className="shell">
      <header className="masthead">
        <div className="identity"><span className="mark" aria-hidden="true">D<span>·</span>C</span><span>Deep Create</span></div>
        <span className="build-label">BUILD INFORMATION</span>
      </header>

      <section className="intro" aria-labelledby="page-title">
        <p className="eyebrow">STAGE 1 · APPLICATION FOUNDATION</p>
        <h1 id="page-title">A standalone shell.<br /><span>Execution is still ahead.</span></h1>
        <p className="lead">Deep Create cannot execute work yet. This build displays its version and work unit list — without models, executors, or user data.</p>
      </section>

      <section className="version-panel" aria-labelledby="version-title">
        <div className="panel-heading"><span className="small-square" aria-hidden="true" /><h2 id="version-title">Current build</h2><span className="read-only">Read only</span></div>
        <Metadata state={metadata} />
        <p className="source-note">The version is read from the native application. This page does not establish system readiness.</p>
      </section>

      <ObjectiveForm
        onPlan={handlePlan}
        onSubmit={handleSubmit}
        onAccept={handleAcceptGraph}
        onExecute={handleExecute}
        planState={planState}
        writeState={writeState}
      />

      <section className="version-panel" aria-labelledby="work-graph-title">
        <div className="panel-heading"><span className="small-square" aria-hidden="true" /><h2 id="work-graph-title">Work graph</h2><span className="read-only">Read only</span></div>
        <p className="fixture-banner" role="note">Демонстрационный набор данных. Не является актуальным состоянием системы.</p>
        <WorkUnits state={workUnits} onStartAttempt={handleStartAttempt} writeState={writeState} />
        <p className="source-note">The work unit list is read from the native application. This does not confirm execution.</p>
      </section>

      <section className="boundaries" aria-labelledby="boundaries-title">
        <h2 id="boundaries-title">Limits of this version</h2>
        <div className="boundary-grid">
          <article><span className="boundary-number" aria-hidden="true">01</span><h3>No data access</h3><p>The window has no permissions for files, the database, the command line, or external networking.</p></article>
          <article><span className="boundary-number" aria-hidden="true">02</span><h3>No hidden work</h3><p>No models are running, background executors, or simulated progress.</p></article>
          <article><span className="boundary-number" aria-hidden="true">03</span><h3>Separate from Hermes</h3><p>The application does not connect to the Hermes profile, keys, or state.</p></article>
        </div>
      </section>

      <footer><span>Local Windows application</span><span>Capabilities are added only after the foundation has been verified.</span></footer>
    </main>
  );
}
